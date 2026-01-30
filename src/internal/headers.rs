use super::request::ErrorMsg;
use core::str;
use std::collections::hash_map::Iter;
// use std::str::FromStr;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

const CRLF: &[u8; 2] = b"\r\n";
// const SP: u8 = b' ';

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Headers {
    inner: HashMap<String, String>,
}
impl Headers {
    fn new() -> Headers {
        return Headers {
            inner: HashMap::new(),
        };
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.inner.get(&k.to_lowercase()).map(|v| v.as_str())
    }

    fn set(&mut self, k: &str, v: &str) {
        let key_lower = k.to_lowercase();

        self.inner
            .entry(key_lower)
            .and_modify(|existing_value| {
                existing_value.push_str(", ");
                existing_value.push_str(v);
            })
            .or_insert_with(|| v.to_string());
    }

    pub fn iter(&self) -> Iter<String, String> {
        self.inner.iter()
    }
}

fn is_token_char(byte: &u8) -> bool {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => true,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_'
        | b'`' | b'|' | b'~' => true,
        _ => false,
    }
}
pub fn parse_field_lines(bytes: &[u8]) -> Result<(Headers, usize), Error> {
    // Field line syntax -> field-name: field-value
    //
    // RULES
    // There should no whitespace between the field name and :
    // A field-value can have OWS as prefix and suffix

    let mut headers = Headers::new();
    let mut bytes_to_read: &[u8] = bytes;
    let mut read: usize = 0;

    loop {
        let field_line_idx = match bytes_to_read.windows(CRLF.len()).position(|b| b == CRLF) {
            Some(idx) => {
                if idx == 0 {
                    return Ok((headers, read));
                }
                idx
            }
            None => break,
        };
        let field_line = &bytes_to_read[0..field_line_idx];

        let mut x = field_line.splitn(2, |b| *b == b':');
        let field_name = x.next().ok_or(Error::new(
            ErrorKind::InvalidData,
            ErrorMsg::INVALID_FIELD_LINE,
        ))?;
        let field_value = x.next().ok_or(Error::new(
            ErrorKind::InvalidData,
            ErrorMsg::INVALID_FIELD_LINE,
        ))?;

        // .all breaks if it encounters a false value
        let is_valid_field_name = field_name.iter().all(is_token_char);
        if !is_valid_field_name {
            println!(
                "Invalid field name {:?}",
                String::from_utf8_lossy(field_name)
            );
            return Err(Error::new(
                ErrorKind::InvalidData,
                ErrorMsg::INVALID_FIELD_LINE,
            ));
        }

        // let is_valid_field_value = field_value.iter().all(|b| b.is_ascii_digit)
        let bytes_to_string = |bytes: &[u8]| -> Result<String, Error> {
            str::from_utf8(bytes)
                .map(|m| m.to_string())
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in headers"))
        };
        let field_name_name_str = bytes_to_string(field_name)?;
        let field_name_value_str = bytes_to_string(field_value)?;

        headers.set(
            &field_name_name_str.to_lowercase(),
            &field_name_value_str.trim().to_string(),
        );

        read += field_line_idx + CRLF.len();
        bytes_to_read = &bytes_to_read[field_line_idx + CRLF.len()..];
    }
    println!("DONE => {:?}", headers);
    Ok((headers, read))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_field_lines() {
        let mut input: &[u8] = b"Authorization: mytoken \r\nContent-type: application/json \r\n";

        let (headers, read) = parse_field_lines(input).unwrap();
        let authorization = headers.get("authorization");
        assert_eq!(authorization, Some("mytoken"));
        println!("After firs");

        let content_type = headers.get("content-type");
        assert_eq!(content_type, Some("application/json"));
        assert_eq!(read, 58);

        input = b" Authorization: my token \r\nContent-type: application/json \r\n";
        let mut x = parse_field_lines(input);
        let mut error = x.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);

        input = b"A/uthorization: my token \r\nContent-type: application/json \r\n";
        x = parse_field_lines(input);
        error = x.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}
