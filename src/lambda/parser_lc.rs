use std::collections::HashMap;

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
use thiserror::Error;

use super::evaluator::Term;

// TODO: add better error messages after replacing nom
#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("something happened during preprocessing")]
    Preprocess,
    #[error("something happened during parsing")]
    Final,
}

pub fn parse(s: &str) -> Result<Term, ParseError> {
    let (_, prog) = preprocess(s).map_err(|_| ParseError::Preprocess)?;

    let mut env = Vec::new();
    lambda(&mut env, &prog)
        .map(|(_, res)| res)
        .map_err(|_| ParseError::Final)
}

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

type Env<'a> = Vec<&'a str>;

fn var<'a>(env: &mut Env<'a>, s_original: &'a str) -> IResult<&'a str, Term> {
    let (s, key) = terminated(identifier, multispace0)(s_original)?;
    if let Some(i) = env.iter().rev().position(|&x| x == key) {
        let i = u32::try_from(i).expect("cast overflow");
        return Ok((s, Term::Var(i)));
    }

    // TODO: nom has awful errors, so please use a different parsing library
    Err(nom::Err::Failure(Error::new(
        s,
        nom::error::ErrorKind::Fail,
    )))
}

fn lambda<'a>(env: &mut Env<'a>, s: &'a str) -> IResult<&'a str, Term> {
    let (s, _) = terminated(char('\\'), multispace0)(s)?;
    let (s, key) = terminated(identifier, multispace0)(s)?;
    let (s, _) = terminated(char('.'), multispace0)(s)?;

    env.push(key);
    let (s, body) = expr(env)(s)?;
    env.pop();

    Ok((s, Term::Lam(Box::new(body))))
}

fn term<'a, 'b>(env: &'b mut Env<'a>) -> impl FnMut(&'a str) -> IResult<&'a str, Term> + 'b {
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

fn expr<'a, 'b>(env: &'b mut Env<'a>) -> impl FnMut(&'a str) -> IResult<&'a str, Term> + 'b {
    |s| {
        let (s, mut terms) = many1(term(env))(s)?;
        let expr = terms
            .drain(..)
            .reduce(|a, b| Term::App(Box::new((a, b))))
            .expect("many1 already asserts that this is Some, so you shouldn't see this");

        Ok((s, expr))
    }
}
