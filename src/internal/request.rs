use super::headers::{Headers, parse_field_lines};
use core::str;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

const CRLF: &[u8; 2] = b"\r\n";
const SP: u8 = b' ';

pub struct ErrorMsg;

impl ErrorMsg {
    pub const MALFROMED_START_LINE: &str = "malformed start line";
    pub const UNSUPPORTED_METHOD: &str = "This request method is not implemented.";
    pub const INVALID_HTTP_SPECIFICATION: &str = "Invalid HTTP specification.";
    pub const INVALID_HTTP_VERSION: &str = "Invalid HTTP version.";
    pub const INVALID_FIELD_LINE: &str = "Invalid field line.";
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

            _ => Err(Error::new(ErrorKind::Unsupported, ErrorMsg::UNSUPPORTED_METHOD).to_string()),
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
    pub headers: Option<Headers>,
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
            headers: None,
            body: vec![],
        }
    }

    fn is_done(&self) -> bool {
        self.state == ParsingState::Error || self.state == ParsingState::Done
    }
}

// Following the RFC 9112
pub fn parse(request_data: &[u8]) -> Result<Request, Error> {
    // cursor
    let mut read: usize = 0;

    // STEPS ON HOW TO PARSE A MESSAGE
    // Parse the start line first
    // parse the field lines into a hash table
    // check the parsed data if there is a body required

    let mut request = Request::new();
    loop {
        let idx = match request_data[read..]
            .windows(CRLF.len())
            .position(|r| r == CRLF)
        {
            Some(i) => i,
            None => break,
        };

        let end_of_line = read + idx;
        let curr_data = &request_data[read..end_of_line];

        println!("current state => {:?}", request.state.as_str());
        match request.state {
            ParsingState::Init => {
                match parse_request_line(curr_data) {
                    Ok((m, t, v, bytes_read)) => {
                        println!("==== Request line ==== ");

                        request.method = Some(m);
                        request.path = Some(t);
                        request.version = Some(v);
                        request.state = ParsingState::Header;
                        read += bytes_read;

                        println!("- Method: {:?}", request.method);
                        println!("- Path: {:?}", request.path);
                        println!("- Version: {:?}", request.version);
                    }
                    Err(e) => {
                        eprintln!("!! Error: {}", e);
                        request.state = ParsingState::Error;
                        return Err(e);
                    }
                };
            }
            ParsingState::Header => {
                println!("XXX");
                let (headers, bytes_read) = parse_field_lines(&request_data[read..])?;

                println!("Headers");
                for x in headers.iter() {
                    println!(" - {}: {}", x.0, x.1);
                }

                request.headers = Some(headers);
                request.state = ParsingState::Body;

                println!("===========");
                read += bytes_read;
            }
            ParsingState::Body => {
                println!("Incomint");
                break;
            }
            ParsingState::Error => todo!(),
            ParsingState::Done => todo!(),
        }

        println!("Total read = {read}");

        // read the \r\n
    }

    println!("DONE");
    Ok(request)
}

fn parse_request_line(b: &[u8]) -> Result<(RequestMethod, String, String, usize), Error> {
    // split by white space
    // result = [method SP request-target SP HTTP-version]
    // since the bytes are in UTF-8 - we have to normalize them to strings

    // Task - remove .collect() for reduce allocation
    let mut read: usize = 0;

    let x: Vec<&[u8]> = b.split(|e| *e == SP).collect();
    println!("after split => {x:?}");
    if x.len() != 3 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            ErrorMsg::MALFROMED_START_LINE,
        ));
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
                ErrorMsg::INVALID_HTTP_SPECIFICATION,
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
        return Err(Error::new(
            ErrorKind::InvalidInput,
            ErrorMsg::INVALID_HTTP_VERSION,
        ));
    }

    read += b.len();
    read += 2;
    println!("total read in start line => {}", read);
    Ok((request_method, target, version, read))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let mut input: &[u8] = b"GET / HTTP/1.1";

        let (m, t, v, bytes_read) = parse_request_line(&input).unwrap();

        assert_eq!(m, RequestMethod::Get);
        assert_eq!(t, "/");
        assert_eq!(v, "HTTP/1.1");
        assert_eq!(bytes_read, 16);

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
