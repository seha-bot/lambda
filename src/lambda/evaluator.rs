use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Clone)]
pub enum Term {
    Var(Rc<RefCell<Option<Term>>>),
    Lam(Rc<RefCell<Option<Term>>>, Box<Term>),
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

    // I'm pretty sure this can be written without recursion
    // Fuck that. Can this function be removed completely?
    fn eval_var_refs(&mut self) {
        match self {
            Term::Var(x) => {
                let expr = (*x).borrow().clone();
                if let Some(expr) = expr {
                    *self = expr;
                }
            }
            Term::Lam(var, body) => {
                let x = (*var).borrow_mut().take();
                body.eval_var_refs();
                *(*var).borrow_mut() = x;
            }
            Term::App(app) => {
                app.0.eval_var_refs();
                app.1.eval_var_refs();
            }
        };
    }

    fn eval_one(self) -> (Term, bool) {
        match self {
            var @ Term::Var(_) => (var, false),
            lam @ Term::Lam(_, _) => (lam, false),
            Term::App(app) => {
                let (f, has_changed) = app.0.eval_one();
                let x = app.1;
                if has_changed {
                    return (Term::App(Box::new((f, x))), true);
                }
                if let Term::Lam(var, mut body) = f {
                    *(*var).borrow_mut() = Some(x);
                    body.eval_var_refs();
                    (*body, true)
                } else {
                    (Term::App(Box::new((f, x))), false)
                }
            }
        }
    }
}
