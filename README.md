# UselessDB

<pre>
___________________
< Don't use me ever >
-------------------
       \   ^__^
        \  (oo)\_______
           (__)\       )\/\
               ||----w |
               ||     ||
</pre>

Prepare yourself for the ultimate regret.  This is a statically typed single value database.  

"Do one thing and do it well."

## FAQ

### I need more than 1 variable!

Run one version of the DB for each variable you need.

### Isn't that a bad idea?

Yes.  Don't use this database for anything.  If you do, you should be fired.

## Examle Usage (client is prefixed with > only to show commands)

    telnet localhost 6000
    > var = 1
    type_error
    > type = int
    ok
    > var = 2
    ok
    > var = 2.0
    type_error
    > var = 5
    ok
    > var > 4
    true
    > var < 10
    true
    > var >= 5
    true
