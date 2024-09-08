# NOTE (for me)

Don't forget about output-fmt. You disabled it to refactor things and probably forgot to enable it.

# DEFECTS

This compiles fine.
Should macros be checked?
```
ID = \x.y;
\y.ID;
```

---

Can't add execution policy.

Proposal: lazy, eager, passthrough.

I still don't know if lazy and eager should be separate concepts because my implementation is shite.

---

Named parameter output format missing.
De bruijn sucks for reading.

---

You forgot to make macro invocations prefixed with a # symbol or something else dumbass.

---

Everything after "main" is a comment is ignored by the parser.

---
