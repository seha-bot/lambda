use const_format::formatcp;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("found a value which is neither true nor false")]
    NonBooleanValue,
    #[error("found a list which doesn't end with NIL")]
    UndelimitedList,
}

pub fn bytes_to_blc(buf: &[u8]) -> String {
    let data = buf.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(PAIR_BLANK);
        acc.push_str(&byte_to_blc(*x));
        acc
    });

    format!("{data}{PAIR_END}")
}

pub fn blc_to_byte(mut data: &str) -> Result<(u8, usize), ParseError> {
    let mut x = 0;
    let mut bytes_read = 0;

    for _ in 0..8 {
        x <<= 1;
        if data.starts_with(PAIR_TRUE) {
            x |= 1;
            data = &data[PAIR_TRUE.len()..];
            bytes_read += PAIR_TRUE.len();
        } else if data.starts_with(PAIR_FALSE) {
            data = &data[PAIR_FALSE.len()..];
            bytes_read += PAIR_FALSE.len();
        } else {
            return Err(ParseError::NonBooleanValue);
        }
    }

    if data.starts_with(PAIR_END) {
        Ok((x, bytes_read + PAIR_END.len()))
    } else {
        Err(ParseError::UndelimitedList)
    }
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
