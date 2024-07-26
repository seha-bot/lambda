extern crate alloc;
mod core;

use core::run;

fn main() {
    match run("0100100010") {
        Ok(out) => println!("{out}"),
        Err(err) => println!("ERROR: {err:?}"),
    }
}
