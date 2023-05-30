use std::{
    ffi::{c_int, c_uint, CStr},
    os::unix::process::CommandExt,
    process::{Command, Output},
    sync::Arc,
};

use anyhow::{bail, Result};
use libc::zoneid_t;

/*
 * ctr_params flags
 */
const CT_PR_PGRPONLY: c_uint = 0x4;
const CT_PR_REGENT: c_uint = 0x8;

/*
 * ctr_ev_* flags
 */
const CT_PR_EV_HWERR: c_uint = 0x20;

#[link(name = "contract")]
extern "C" {
    fn ct_tmpl_set_critical(fd: c_int, events: c_uint) -> c_int;
    fn ct_tmpl_set_informative(fd: c_int, events: c_uint) -> c_int;
    fn ct_pr_tmpl_set_fatal(fd: c_int, events: c_uint) -> c_int;
    fn ct_pr_tmpl_set_param(fd: c_int, params: c_uint) -> c_int;
    fn ct_tmpl_activate(fd: c_int) -> c_int;
    fn ct_tmpl_clear(fd: c_int) -> c_int;
}

#[link(name = "c")]
extern "C" {
    fn zone_enter(zid: zoneid_t) -> c_int;
}

struct Template {
    fd: c_int,
}

impl Drop for Template {
    fn drop(&mut self) {
        if unsafe { libc::close(self.fd) } != 0 {
            let e = std::io::Error::last_os_error();
            panic!("close of ctfs template failed: {e}");
        }
    }
}

impl Template {
    fn new() -> Result<Template> {
        let path =
            CStr::from_bytes_with_nul(b"/system/contract/process/template\0")
                .unwrap();
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            let e = std::io::Error::last_os_error();
            bail!("failed to open contract process template: {e}");
        }

        /*
         * This is lifted from init_template() in the implementation of zlogin,
         * but is using things documented in contract(5) and libcontract(3LIB)
         * etc.
         */
        if unsafe { ct_tmpl_set_critical(fd, 0) } != 0
            || unsafe { ct_tmpl_set_informative(fd, 0) } != 0
            || unsafe { ct_pr_tmpl_set_fatal(fd, CT_PR_EV_HWERR) } != 0
            || unsafe {
                ct_pr_tmpl_set_param(fd, CT_PR_PGRPONLY | CT_PR_REGENT)
            } != 0
            || unsafe { ct_tmpl_activate(fd) } != 0
        {
            let e = std::io::Error::last_os_error();
            bail!("contract template creation failure: {e}");
        }

        Ok(Template { fd })
    }

    fn clear(&self) {
        unsafe { ct_tmpl_clear(self.fd) };
    }
}

fn mkcmd(
    zid: Option<zoneid_t>,
    tmpl: Option<Arc<Template>>,
) -> Command {
    let mut cmd = Command::new("/usr/bin/bash");
    cmd.env_clear();
    cmd.arg("-c");
    cmd.arg("echo in zone $(zonename); ptree $$; echo; pargs $$; echo");
    cmd.arg("--");
    cmd.arg("one");
    cmd.arg("two three");
    cmd.arg("\" four \"");

    if let Some(zid) = zid {
        unsafe {
            cmd.pre_exec(move || {
                if let Some(tmpl) = &tmpl {
                    /*
                     * Clear the contract template before we go on to fork more
                     * processes inside the zone.
                     */
                    tmpl.clear();
                }

                /*
                 * This is technically a private system call, but is also
                 * extremely unlikely to change at this late stage.
                 */
                if zone_enter(zid) != 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
    }

    cmd
}

fn assess(out: Output) -> Result<()> {
    if !out.status.success() {
        bail!("command failed");
    }
    println!(
        "{}",
        String::from_utf8(out.stdout)?
            .lines()
            .map(|l| format!("    | {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

fn main() -> Result<()> {
    println!(" * in gz...");
    let out = mkcmd(None, None).output();
    assess(out?)?;

    let zid = 1;
    println!(" * in zone {zid}...");

    /*
     * The child process needs to be divorced from the contract of the current
     * process before it enters the zone.  Create a contract template and
     * activate it for the current thread before we fork.
     */
    let t = Arc::new(Template::new()?);
    let out = mkcmd(Some(zid), Some(Arc::clone(&t))).output();
    t.clear();
    assess(out?)?;

    Ok(())
}
