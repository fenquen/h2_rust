use anyhow::Result;
use lazy_static::lazy_static;
use crate::api::error_code;
use crate::message::db_error::DbError;
use crate::throw;
use crate::h2_rust_common::Integer;

lazy_static! {
     static ref HEX_DECODE:[Integer;104] ={
        let mut r:[Integer;104] = [-1;104];

        for a in 0..10{
            r[a + 48] = a as Integer;
        }

        for a in 0..6{
            r[a + 97] = (a + 10) as Integer;
            r[a + 65] = (a + 10) as Integer;
        }

        r
     };
}

pub fn convert_hex_to_byte_vec(s: &str) -> Result<Vec<u8>> {
    let mut len = s.len();
    if len % 2 != 0 {
        throw!(DbError::get(error_code::HEX_STRING_ODD_1, vec![s]));
    }

    len = len / 2;

    let mut buff = Vec::with_capacity(len);
    let mut mask: Integer = 0;


    let mut chars = s.chars();
    for l in 0..len {
        let a = chars.next().unwrap() as usize;
        let b = chars.next().unwrap() as usize;

        let d = HEX_DECODE[a] << 4 | HEX_DECODE[b];
        mask = mask | d;
        buff[l] = d as u8;
    }

    if mask & !(255 as Integer) != 0 {
        throw!(DbError::get(error_code::HEX_STRING_WRONG_1, vec![s]));
    }

    Ok(buff)
}

pub fn to_upper_english(s: &str) -> String {
    s.to_uppercase()
}

pub fn to_lower_english(s: &str) -> String {
    s.to_lowercase()
}

pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() > max_length {
        (&s[..max_length]).to_string()
    } else {
        s.to_string()
    }
}

mod test {
    use crate::h2_rust_common::Integer;

    #[test]
    fn test_char_to_number() {
        for c in "a".chars() {
            println!("{}", c as Integer);
        }
    }
}