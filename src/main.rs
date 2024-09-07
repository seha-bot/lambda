use core::run_expr_experimental;
use std::{env, fs, io, process::exit};

extern crate alloc;
mod core;
mod parser;

fn main() -> Result<(), io::Error> {
    let args = env::args().skip(1).take(2).collect::<Vec<_>>();

    let Some(path) = args.first() else {
        eprintln!("Must specify path to file to be run.");
        exit(1)
    };

    let content = fs::read_to_string(path)?;

    match parser::parse_lc(&content) {
        Ok(expr) => {
            println!("Your program: {expr:?}");
            println!("Your program: {expr}");
            println!("Output:");
            match run_expr_experimental(
                expr,
                args.get(1).map(|x| x.as_str()),
                core::OutputFmt::Parsed,
            ) {
                Ok(out) => println!("{out}"),
                Err(_) => {
                    eprintln!("Argument parsing failed. You shouldn't be seeing this!");
                    exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("An error occured during parsing ({err:?}). Error messages are not implemented yet.");
            exit(1);
        }
    }

    Ok(())
}

// \arg.PAIR (arg (\x.\y.x)) arg;
// \arg. (\a. \b. \f. (a) (b)) (arg (\x.\y.x)) arg
