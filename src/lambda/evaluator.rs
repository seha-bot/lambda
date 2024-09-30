#[derive(Clone)]
pub enum Term {
    Var(u32),
    Lam(Box<Term>),
    App(Box<(Term, Term)>),
}

impl Term {
    pub fn eval(mut self) -> Term {
        let mut has_changed = true;
        while has_changed {
            (self, has_changed) = self.eval_one();
        }
        self
    }

    fn sub_vars(&mut self, val: &Term, depth: u32) {
        match self {
            Term::Var(x) => {
                if let Some(0) = x.checked_sub(depth) {
                    *self = val.clone();
                }
            }
            Term::Lam(body) => {
                body.sub_vars(val, depth + 1);
            }
            Term::App(app) => {
                app.0.sub_vars(val, depth);
                app.1.sub_vars(val, depth);
            }
        };
    }

    fn eval_one(self) -> (Term, bool) {
        match self {
            var @ Term::Var(_) => (var, false),
            lam @ Term::Lam(_) => (lam, false),
            Term::App(app) => {
                let (f, has_changed) = app.0.eval_one();
                let x = app.1;
                if has_changed {
                    return (Term::App(Box::new((f, x))), true);
                }
                if let Term::Lam(mut body) = f {
                    body.sub_vars(&x, 0);
                    (*body, true)
                } else {
                    (Term::App(Box::new((f, x))), false)
                }
            }
        }
    }
}
