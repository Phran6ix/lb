use std::str::{self, FromStr};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

const CRLF: &[u8; 2] = b"\r\n";
const SP: u8 = b' ';

pub struct ErrorMsg;

impl ErrorMsg {
    pub const MALFROMED_START_LINE: &str = "malformed start line";
    pub const UNSUPPORTED_METHOD: &str = "This request method is not implemented.";
    pub const INVALID_HTTP_SPECIFICATION: &str = "Invalid HTTP specification.";
    pub const INVALID_HTTP_VERSION: &str = "Invalid HTTP version.";
}

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    Get,
    Post,
    Patch,
    Delete,
    Put,
}

impl FromStr for RequestMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(RequestMethod::Get),
            "POST" => Ok(RequestMethod::Post),
            "PATCH" => Ok(RequestMethod::Patch),
            "PUT" => Ok(RequestMethod::Put),
            "DELETE" => Ok(RequestMethod::Delete),

            _ => Err(Error::new(
                ErrorKind::Unsupported,
                ErrorMsg::UNSUPPORTED_METHOD,
            )
            .to_string()),
        }
    }
}

#[derive(PartialEq)]
enum ParsingState {
    Init,
    Header,
    Body,
    Error,
    Done,
}

impl ParsingState {
    // To be used for logging purposes only
    fn as_str(&self) -> &str {
        match self {
            ParsingState::Init => "init",
            ParsingState::Header => "header",
            ParsingState::Body => "body",
            ParsingState::Error => "error",
            ParsingState::Done => "done",
        }
    }
}

pub struct Request {
    state: ParsingState,
    pub method: Option<RequestMethod>,
    pub version: Option<String>,
    pub headers: HashMap<String, String>,
    pub path: Option<String>,
    pub body: Vec<u8>,
}

impl Request {
    fn new() -> Self {
        Request {
            state: ParsingState::Init,
            method: None,
            version: None,
            path: None,
            headers: HashMap::new(),
            body: vec![],
        }
    }

    fn is_done(&self) -> bool {
        self.state == ParsingState::Error || self.state == ParsingState::Done
    }
}

// Following the RFC 9112
pub fn process_request(request_data: &[u8]) -> Result<Request, Error> {
    // cursor
    let mut read: usize = 0;

    // STEPS ON HOW TO PARSE A MESSAGE
    // Parse the start line first
    // parse the field lines into a hash table
    // check the parsed data if there is a body required

    loop {
        let idx = match request_data[read..]
            .windows(CRLF.len())
            .position(|r| r == CRLF)
        {
            Some(i) => i,
            None => break,
        };

        let mut request = Request::new();

        let end_of_line = read + idx;
        let curr_data = &request_data[read..end_of_line];

        read = end_of_line;

        match request.state {
            ParsingState::Init => {
                match parse_request_line(curr_data) {
                    Ok((m, t, v)) => {
                        println!("==== Request line ==== ");
                        println!("- Method: {m:?}");
                        println!("- Path: {t}");
                        println!("- Version: {v}");

                        request.method = Some(m);
                        request.path = Some(t);
                        request.version = Some(v);
                        request.state = ParsingState::Header;
                    }
                    Err(e) => {
                        eprintln!("!! Error: {}", e);
                    }
                };
            }
            ParsingState::Header => todo!(),
            ParsingState::Body => todo!(),
            ParsingState::Error => todo!(),
            ParsingState::Done => todo!(),
        }

        // read the \r\n
        read += 2;
    }

    todo!()
}

fn parse_request_line(b: &[u8]) -> Result<(RequestMethod, String, String), Error> {
    // split by white space
    // result = [method SP request-target SP HTTP-version]
    // since the bytes are in UTF-8 - we have to normalize them to strings

    let x: Vec<&[u8]> = b.split(|e| *e == SP).collect();
    println!("after split => {x:?}");
    if x.len() != 3 {
        return Err(Error::new(ErrorKind::InvalidInput, ErrorMsg::MALFROMED_START_LINE));
    }

    let bytes_to_strings = |bytes: &[u8], name: &str| -> Result<String, Error> {
        str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|_| Error::new(ErrorKind::InvalidInput, format!("Invalid UTF-8 in {name}")))
    };

    let method = bytes_to_strings(x[0], "method")?;
    let target = bytes_to_strings(x[1], "target")?;
    let version = bytes_to_strings(x[2], "version")?;

    let request_method = match RequestMethod::from_str(&method.as_str()) {
        Ok(mtd) => mtd,
        Err(e) => return Err(Error::new(ErrorKind::Unsupported, e)),
    };

    let p_v: (&str, &str) = match version.split_once("/") {
        Some(p) => p,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                ErrorMsg::INVALID_HTTP_SPECIFICATION
            ));
        }
    };

    if p_v.0 != "HTTP" && p_v.0 != "HTTPS" {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            ErrorMsg::INVALID_HTTP_SPECIFICATION,
        ));
    }

    if p_v.1 != "1.1" && p_v.1 != "1.0" {
        return Err(Error::new(ErrorKind::InvalidInput, ErrorMsg::INVALID_HTTP_VERSION));
    }

    Ok((request_method, target, version))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let mut input: &[u8] = b"GET / HTTP/1.1";

        let (m, t, v) = parse_request_line(&input).unwrap();

        assert_eq!(m, RequestMethod::Get);
        assert_eq!(t, "/");
        assert_eq!(v, "HTTP/1.1");

        input = b"HOST /helllo HTTP/1.1";
        let mut result = parse_request_line(&input);
        let mut error = result.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(error.to_string(), "This request method is not implemented.");

        input = b"POST HTTP/1.1";
        result = parse_request_line(&input);
        error = result.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), ErrorMsg::MALFROMED_START_LINE);

        input = b"PATCH /hello Http/1.1";
        result = parse_request_line(&input);
        error = result.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), ErrorMsg::INVALID_HTTP_SPECIFICATION);

        input = b"PATCH /hello Http 1.1";
        result = parse_request_line(&input);
        error = result.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), ErrorMsg::MALFROMED_START_LINE);

        input = b"PATCH /hello HTTP/2.1";
        result = parse_request_line(&input);
        error = result.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), ErrorMsg::INVALID_HTTP_VERSION);

        input = b"PATCH /hello HTTP/1.1";
        let result = parse_request_line(&input).unwrap();

        assert_eq!(result.0, RequestMethod::Patch);
        assert_eq!(result.1, "/hello");
        assert_eq!(result.2, "HTTP/1.1");
    }
}
