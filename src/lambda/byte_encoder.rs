use const_format::formatcp;

pub fn bytes_to_blc(buf: &[u8]) -> String {
    let data = buf.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(PAIR_BLANK);
        acc.push_str(&byte_to_blc(*x));
        acc
    });

    format!("{data}{PAIR_END}")
}

const PAIR_BLANK: &str = "00010110";
const PAIR_END: &str = "000010";
const TRUE: &str = "0000110";
const FALSE: &str = "000010";
const PAIR_TRUE: &str = formatcp!("{PAIR_BLANK}{TRUE}");
const PAIR_FALSE: &str = formatcp!("{PAIR_BLANK}{FALSE}");

fn byte_to_blc(mut x: u8) -> String {
    let mut data = [false; 8];

    for i in 0..8 {
        data[7 - i] = x % 2 == 1;
        x /= 2;
    }

    let data = data.iter().fold(String::new(), |mut acc, &x| {
        acc.push_str(if x { PAIR_TRUE } else { PAIR_FALSE });
        acc
    });

    format!("{data}{PAIR_END}")
}
