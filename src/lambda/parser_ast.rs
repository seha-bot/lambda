use thiserror::Error;

use super::evaluator::Term;

#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("found a value which is neither true nor false")]
    NonBooleanValue,
    #[error("found a list which doesn't end with NIL")]
    UndelimitedList,
    #[error("program should evaluate to a list or nil: \\f.f char tail || \\f.\\y.y: abstraction for f not found")]
    ExpectedLamForPair,
    #[error("program should evaluate to a list: \\f.f char tail: tail not found")]
    ExpectedAppTail,
    #[error("program should evaluate to a list or nil: \\f.f char tail || \\x.\\y.y")]
    ExpectedAppOrNil,
    #[error("program end should evaluate to a nil: \\x.\\y.y: term y not found")]
    ExpectedVar,
    #[error(
        "program end should evaluate to a nil: \\x.\\y.y: term y not pointing to correct parameter"
    )]
    BadVar,
    #[error("each output char should have 8 bits")]
    ListTerminatedTooEarly,
}

// mangled(\f.f a b) -> (a, b) | Nil
pub fn uncons(term: Term) -> Result<Option<(Term, Term)>, ParseError> {
    let Term::Lam(body) = term else {
        return Err(ParseError::ExpectedLamForPair);
    };

    match body.eval() {
        Term::App(app) => {
            let (mangled_a, b) = *app;

            let Term::App(app) = mangled_a else {
                return Err(ParseError::ExpectedAppTail);
            };

            let (f_var, a) = *app;

            let Term::Var(f_var) = f_var else {
                panic!("make me into a proper error");
            };
            assert_eq!(f_var, 0);

            Ok(Some((a, b)))
        }
        Term::Lam(body) => {
            let Term::Var(y_var) = body.eval() else {
                return Err(ParseError::ExpectedVar);
            };

            if y_var == 0 {
                Ok(None)
            } else {
                Err(ParseError::BadVar)
            }
        }
        Term::Var(_) => Err(ParseError::ExpectedAppOrNil),
    }
}

// TODO: the error can be more detailed
pub fn ast_to_bool(term: Term) -> Result<bool, ParseError> {
    let Term::Lam(body) = term else {
        return Err(ParseError::NonBooleanValue);
    };

    let Term::Lam(body) = body.eval() else {
        return Err(ParseError::NonBooleanValue);
    };

    let Term::Var(x) = body.eval() else {
        return Err(ParseError::NonBooleanValue);
    };

    if x == 1 {
        Ok(true)
    } else if x == 0 {
        Ok(false)
    } else {
        Err(ParseError::NonBooleanValue)
    }
}

pub fn ast_to_byte(mut term: Term) -> Result<u8, ParseError> {
    let mut x = 0;

    for _ in 0..8 {
        x <<= 1;

        let Some((head, tail)) = uncons(term)? else {
            return Err(ParseError::ListTerminatedTooEarly);
        };

        if ast_to_bool(head.eval())? {
            x |= 1;
        }

        term = tail.eval();
    }

    if ast_to_bool(term)? {
        Err(ParseError::UndelimitedList)
    } else {
        Ok(x)
    }
}
