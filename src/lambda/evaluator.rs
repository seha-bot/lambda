#[derive(Debug, Clone)]
pub enum Term {
    Var(u32),
    Lam(u32, Box<Term>),
    App(Box<(Term, Term)>),
}

pub type Env = Vec<(u32, Binding)>;

#[derive(Debug, Clone)]
pub struct Binding(Env, Term);

#[derive(Debug, Clone)]
pub enum BoundTerm {
    Var(u32),
    Lam(Env, u32, Term),
    App(Box<(BoundTerm, Binding)>),
}

impl Binding {
    pub fn eval(self) -> BoundTerm {
        eval(self.0, self.1)
    }
}

pub fn eval(env: Env, term: Term) -> BoundTerm {
    match term {
        Term::Var(i) => match env.iter().rev().find(|(x, _)| *x == i) {
            Some((_, x)) => eval(x.0.clone(), x.1.clone()),
            None => BoundTerm::Var(i),
        },
        Term::Lam(i, body) => BoundTerm::Lam(env, i, *body),
        Term::App(app) => {
            let (f, x) = (eval(env.clone(), app.0), Binding(env, app.1));

            if let BoundTerm::Lam(mut env1, i, body) = f {
                env1.push((i, x));
                eval(env1, body)
            } else {
                BoundTerm::App(Box::new((f, x)))
            }
        }
    }
}
