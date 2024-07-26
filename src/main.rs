extern crate alloc;

use core::run;

mod core;

fn main() {
    match run("0100100010") {
        Ok(out) => println!("{out}"),
        Err(err) => println!("ERROR: {err:?}"),
    }
}
