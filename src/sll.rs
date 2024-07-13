use crate::core::{Expression, Lambda, Substitution, Term};

pub fn id() -> Lambda {
    let x = Substitution::new("x");
    Lambda {
        substitution: x.clone(),
        body: Term::Substitution(x),
    }
}

pub fn zero() -> Lambda {
    Lambda {
        substitution: Substitution::new("f"),
        body: Term::Lambda(Box::new(id())),
    }
}

pub fn num(mut n: u8) -> Lambda {
    let f = Substitution::new("f");
    let x = Substitution::new("x");

    let mut accum = Term::Substitution(x.clone());
    while n > 0 {
        accum = Term::Expression(Expression {
            terms: vec![Term::Substitution(f.clone()), accum],
        });
        n -= 1;
    }

    Lambda {
        substitution: f,
        body: Term::Lambda(Box::new(Lambda {
            substitution: x,
            body: accum,
        })),
    }
}

pub fn inc() -> Lambda {
    let n = Substitution::new("n");
    let f = Substitution::new("f");
    let x = Substitution::new("x");
    Lambda {
        substitution: n.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: f.clone(),
            body: Term::Lambda(Box::new(Lambda {
                substitution: x.clone(),
                body: Term::Expression(Expression {
                    terms: vec![
                        Term::Substitution(f.clone()),
                        Term::Expression(Expression {
                            terms: vec![
                                Term::Substitution(n),
                                Term::Substitution(f),
                                Term::Substitution(x),
                            ],
                        }),
                    ],
                }),
            })),
        })),
    }
}

pub fn plus() -> Lambda {
    let x = Substitution::new("x");
    let y = Substitution::new("y");
    Lambda {
        substitution: x.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: y.clone(),
            body: Term::Expression(Expression {
                terms: vec![
                    Term::Substitution(x),
                    Term::Lambda(Box::new(inc())),
                    Term::Substitution(y),
                ],
            }),
        })),
    }
}

pub fn flip() -> Lambda {
    let y = Substitution::new("y");
    let x = Substitution::new("x");
    Lambda {
        substitution: y.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: x.clone(),
            body: Term::Expression(Expression {
                terms: vec![Term::Substitution(x), Term::Substitution(y)],
            }),
        })),
    }
}

pub fn r#true() -> Lambda {
    let x = Substitution::new("x");
    Lambda {
        substitution: x.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: Substitution::new("y"),
            body: Term::Substitution(x),
        })),
    }
}

pub fn r#false() -> Lambda {
    let y = Substitution::new("y");
    Lambda {
        substitution: Substitution::new("x"),
        body: Term::Lambda(Box::new(Lambda {
            substitution: y.clone(),
            body: Term::Substitution(y),
        })),
    }
}

pub fn pair() -> Lambda {
    let x = Substitution::new("x");
    let y = Substitution::new("y");
    let f = Substitution::new("f");
    Lambda {
        substitution: x.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: y.clone(),
            body: Term::Lambda(Box::new(Lambda {
                substitution: f.clone(),
                body: Term::Expression(Expression {
                    terms: vec![
                        Term::Substitution(f),
                        Term::Substitution(x),
                        Term::Substitution(y),
                    ],
                }),
            })),
        })),
    }
}

pub fn fst() -> Lambda {
    let p = Substitution::new("p");
    Lambda {
        substitution: p.clone(),
        body: Term::Expression(Expression {
            terms: vec![Term::Substitution(p), Term::Lambda(Box::new(r#true()))],
        }),
    }
}

pub fn snd() -> Lambda {
    let p = Substitution::new("p");
    Lambda {
        substitution: p.clone(),
        body: Term::Expression(Expression {
            terms: vec![Term::Substitution(p), Term::Lambda(Box::new(r#false()))],
        }),
    }
}

pub fn compose() -> Lambda {
    let f = Substitution::new("f");
    let g = Substitution::new("g");
    let x = Substitution::new("x");
    Lambda {
        substitution: f.clone(),
        body: Term::Lambda(Box::new(Lambda {
            substitution: g.clone(),
            body: Term::Lambda(Box::new(Lambda {
                substitution: x.clone(),
                body: Term::Expression(Expression {
                    terms: vec![
                        Term::Substitution(f),
                        Term::Expression(Expression {
                            terms: vec![Term::Substitution(g), Term::Substitution(x)],
                        }),
                    ],
                }),
            })),
        })),
    }
}
