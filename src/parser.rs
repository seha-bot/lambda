use std::{cell::RefCell, collections::HashMap, rc::Rc};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::recognize,
    error::Error,
    multi::{many0_count, many1},
    sequence::{delimited, pair, terminated},
    IResult,
};

use crate::core::Expr;

fn identifier(s: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(s)
}

fn body_terminated_raw(s: &str) -> IResult<&str, &str> {
    terminated(
        take_while1(|x| x != ';'),
        terminated(char(';'), multispace0),
    )(s)
}

fn macro_definition_raw(s: &str) -> IResult<&str, (&str, &str)> {
    let (s, key) = identifier(s)?;
    let (s, _) = delimited(multispace0, char('='), multispace0)(s)?;
    let (s, body) = body_terminated_raw(s)?;

    Ok((s, (key, body)))
}

fn apply_macros(macros: &Macros, body: &str) -> String {
    let mut body = String::from(body);
    for (key, val) in macros {
        body = body.replace(key, &format!("({val})"));
    }

    body
}

type Macros<'a> = HashMap<&'a str, String>;
fn macro_definitions(mut s: &str) -> (&str, Macros) {
    let mut macros = HashMap::new();
    while let Ok((new_s, (key, body))) = macro_definition_raw(s) {
        macros.insert(key, apply_macros(&macros, body));
        s = new_s;
    }

    (s, macros)
}

fn preprocess(s: &str) -> IResult<&str, String> {
    let (s, macros) = macro_definitions(s);
    let (s, body) = body_terminated_raw(s)?;
    let body = apply_macros(&macros, body);
    Ok((s, body))
}

type Env<'a> = HashMap<&'a str, Rc<RefCell<Option<Expr>>>>;

fn var<'a, 'b>(env: &'b mut Env<'a>, s: &'a str) -> IResult<&'a str, Expr> {
    let (s, key) = terminated(identifier, multispace0)(s)?;
    if let Some(key_ref) = env.get(key) {
        return Ok((s, Expr::Var(Rc::clone(key_ref))));
    }

    // TODO: nom has awful errors, so please use a different parsing library
    Err(nom::Err::Failure(Error::new(
        s,
        nom::error::ErrorKind::Fail,
    )))
}

fn lambda<'a, 'b>(env: &'b mut Env<'a>, s: &'a str) -> IResult<&'a str, Expr> {
    let (s, _) = terminated(char('\\'), multispace0)(s)?;
    let (s, key) = terminated(identifier, multispace0)(s)?;
    let (s, _) = terminated(char('.'), multispace0)(s)?;

    let key_ref = Rc::new(RefCell::new(None));

    let prev = env.insert(key, Rc::clone(&key_ref));
    let (s, body) = expr(env)(s)?;
    if let Some(prev) = prev {
        env.insert(key, prev);
    } else {
        env.remove(key);
    }

    Ok((s, Expr::Lam(key_ref, Box::new(body))))
}

fn term<'a, 'b>(env: &'b mut Env<'a>) -> impl FnMut(&'a str) -> IResult<&'a str, Expr> + 'b {
    move |s| {
        lambda(env, s).or_else(|_| var(env, s)).or_else(|_| {
            terminated(
                delimited(
                    char('('),
                    delimited(multispace0, expr(env), multispace0),
                    char(')'),
                ),
                multispace0,
            )(s)
        })
    }
}

fn expr<'a, 'b>(env: &'b mut Env<'a>) -> impl FnMut(&'a str) -> IResult<&'a str, Expr> + 'b {
    |s| {
        let (s, mut terms) = many1(term(env))(s)?;
        let expr = terms
            .drain(..)
            .reduce(|a, b| Expr::App(Box::new((a, b))))
            .expect("many1 already asserts that this is Some, so you shouldn't see this");

        Ok((s, expr))
    }
}

// TODO: add better error messages after replacing nom
#[derive(Debug)]
pub enum ParseLCError {
    Preprocess,
    Final,
}

pub fn parse_lc(s: &str) -> Result<Expr, ParseLCError> {
    let (_, prog) = preprocess(s).map_err(|_| ParseLCError::Preprocess)?;

    let mut env = HashMap::new();
    lambda(&mut env, &prog)
        .map(|(_, res)| res)
        .map_err(|_| ParseLCError::Final)
}
