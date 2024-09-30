use std::{fs, io, path::PathBuf, process::exit};

use clap::Parser;
use lambda::{InputFmt, OutputFmt, RunError};

mod lambda;

/// A lambda calculus parser & runner.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
    arg: Option<String>,

    #[arg(long, default_value_t = String::from("standard"))]
    input_fmt: String,

    #[arg(long, default_value_t = String::from("bytes"))]
    output_fmt: String,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let input_fmt = match args.input_fmt.as_str() {
        "binary" => InputFmt::Binary,
        "standard" => InputFmt::Standard,
        other => {
            eprintln!("{other:?} not recognised as an input format.");
            eprintln!("Use one of the following: binary, standard.");
            exit(1)
        }
    };

    let output_fmt = match args.output_fmt.as_str() {
        "bytes" => OutputFmt::Bytes,
        "bits" => OutputFmt::Bits,
        other => {
            eprintln!("{other:?} not recognised as an output format.");
            eprintln!("Use one of the following: bytes, bits.");
            exit(1)
        }
    };

    let content = fs::read_to_string(args.path)?;

    let res = lambda::run(
        &content,
        match &args.arg {
            Some(x) => Some(x.as_str()),
            None => None,
        },
        input_fmt,
        output_fmt,
    );

    if let Err(err) = res {
        if let RunError::IO(_) = err {
        } else {
            eprintln!("{err}");
        }
        exit(1)
    }

    Ok(())
}
