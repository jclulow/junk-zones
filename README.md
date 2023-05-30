# junk-zones

Just a quick demonstration of using `Command::pre_exec()` to enter a zone
before executing a command.  This is roughly analogous to what `zlogin` is
doing.

```
$ pfexec ./target/debug/zones
 * in gz...
    | in zone global
    | 11932  /usr/sbin/sshd
    |   17464  /usr/sbin/sshd -R
    |     17466  /usr/sbin/sshd -R
    |       17467  -bash
    |         18447  ./target/debug/zones
    |           18448  /usr/bin/bash -c echo in zone $(zonename); ptree $$; echo; par
    |             18450  ptree 18448
    |
    | 18448:    /usr/bin/bash -c echo in zone $(zonename); ptree $$; echo; pargs $$; echo -- on
    | argv[0]: /usr/bin/bash
    | argv[1]: -c
    | argv[2]: echo in zone $(zonename); ptree $$; echo; pargs $$; echo
    | argv[3]: --
    | argv[4]: one
    | argv[5]: two three
    | argv[6]: " four "
    |
 * in zone 1...
    | in zone bmat-test
    | 2243   zsched
    |   18452  /usr/bin/bash -c echo in zone $(zonename); ptree $$; echo; pargs $$; e
    |     18454  ptree 18452
    |
    | 18452:    /usr/bin/bash -c echo in zone $(zonename); ptree $$; echo; pargs $$; echo -- on
    | argv[0]: /usr/bin/bash
    | argv[1]: -c
    | argv[2]: echo in zone $(zonename); ptree $$; echo; pargs $$; echo
    | argv[3]: --
    | argv[4]: one
    | argv[5]: two three
    | argv[6]: " four "
    |
```
