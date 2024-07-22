use core::temp_full_parse;

mod core;

fn main() {
    // let prog = "0100100010"; // (\ 1) (\ 1)

    let prog = "010001101000011010"; // (\ 1 1) (\ 1 1)

    let prog = temp_full_parse(prog);
    println!("{prog}");

    prog.eval();
}
