# How to use

`cargo r examples/reverse.lc hello` -> `olleh`

`cargo r -- --input-fmt=binary --output-fmt=bits examples/primes.blc 0` -> your pc will crash :)

`cargo r -- --input-fmt=binary examples/echo.blc "Hello World"` -> `Hello World`

`cargo r examples/repeat.lc 0 | head -c 5` -> `00000`

# DEFECTS

THE INTERPRETER IS SLOW! PLEASE SEEK HELP

---

This compiles fine.
Should macros be checked?
```
ID = \x.y;
\y.ID;
```

---

You forgot to make macro invocations prefixed with a # symbol or something else dumbass.

---

Everything after "main" is a comment is ignored by the parser.

---
