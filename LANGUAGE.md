'\' | 'λ'       -> abstraction start
'.'             -> body start
[a-z]* | [A-Z]* -> variable / function names
"NAME = EXPR;"  -> macro start
'(' & ')'       -> precedence separators

Examples:

macro substitution
```
ID = \x.x;
ID ID
```

---

shadowing
```
ID = \x.x;
ZERO = \x.ID;
ONE = ID;
```
