extern crate alloc;
mod core;

fn run_print(prog: &str, args: Option<&str>, output_fmt: core::OutputFmt) {
    match core::run(prog, args, output_fmt) {
        Ok(out) => println!("{out}"),
        Err(err) => println!("ERROR: {err:?}"),
    }
}

fn main() {
    run_print("0010", Some("Hello World"), core::OutputFmt::Parsed);
    run_print("0100100010", None, core::OutputFmt::HumanReadable);
    run_print("0100100010", None, core::OutputFmt::AsciiBinary);
}
