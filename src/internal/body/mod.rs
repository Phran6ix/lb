use crate::internal::body::{chunked::parse_chunked_message, fixed::parse_fixed_message};

use super::request::Request;
use std::io::{Error, ErrorKind};

mod chunked;
mod fixed;

pub fn parse_request_body(bytes: &[u8], request: &mut Request) -> Result<usize, Error> {
    let header = request
        .headers
        .as_ref()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Headers missing during body parse"))?;

    if let Some(te) = header.get("Transfer-Encoding") {
        if te.to_lowercase().contains("chunked") {
            return parse_chunked_message(bytes, &mut  request.body);
        }
    }

    if let Some(cl) = header.get("Content-Length") {
        let length = cl
            .parse::<usize>()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Content-Length"))?;

        return parse_fixed_message(bytes, length, &mut request.body);
    }

    Ok(0)
}
