use crate::internal::body::parse_request_body;

use super::headers::{Headers, parse_field_lines};
use core::str;
use std::collections::HashMap;
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

impl RequestMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestMethod::Get => "GET",
            RequestMethod::Post => "POST",
            RequestMethod::Patch => "PATCH",
            RequestMethod::Put => "PUT",
            RequestMethod::Delete => "DELETE",
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
    //  for logging purposes only
    // fn as_str(&self) -> &str {
    //     match self {
    //         ParsingState::Init => "init",
    //         ParsingState::Header => "header",
    //         ParsingState::Body => "body",
    //         ParsingState::Error => "error",
    //         ParsingState::Done => "done",
    //     }
    // }
}

pub struct Request {
    state: ParsingState,
    pub method: Option<RequestMethod>,
    pub version: Option<String>,
    pub headers: Option<Headers>,
    pub path: Option<String>,
    pub body: Vec<u8>,
    pub param: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
}

impl Request {
    pub fn new() -> Self {
        Request {
            state: ParsingState::Init,
            method: None,
            version: None,
            path: None,
            headers: None,
            body: vec![],
            param: None,
            query: None,
        }
    }

    fn is_done(&self) -> bool {
        self.state == ParsingState::Error || self.state == ParsingState::Done
    }

    pub fn set_param(&mut self, param: HashMap<String, String>) -> &mut Self {
        if param.is_empty() {
            return self;
        }

        let request_params = self.param.get_or_insert_with(HashMap::new);

        // let Some(req_params) = &mut self.param else {
        //     self.param = Some(param);
        //     return self;
        // };

        request_params.reserve(param.len());
        request_params.extend(param);
        self
    }

    pub fn set_query(&mut self, query: HashMap<String, String>) -> &mut Self {
        if query.is_empty() {
            return self;
        }
        let req_query = self.query.get_or_insert_with(HashMap::new);
        // let Some(req_querys) = &mut self.query else {
        //     self.query = Some(query);
        //     return self;
        // };

        req_query.reserve(query.len());
        req_query.extend(query);
        self
    }

    // Does the reverse of parse function
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        // This will be the buffer that will be streamed
        let mut request_bytes: Vec<u8> = Vec::new();

        // Start lines

        let Some(method) = &self.method else {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid Data"));
        };

        let Some(path) = &self.path else {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid Data"));
        };

        let Some(version) = &self.version else {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid Data"));
        };

    
        let start_line_str = format!("{} {} HTTP/{}", method.as_str(), path, version);

        // Serilizing the headers
        // https://datatracker.ietf.org/doc/html/rfc9112#name-field-syntax

        let mut field_lines: Vec<u8> = Vec::new();

        field_lines.extend_from_slice(b"Forwarded: my_loadbalancer");
        field_lines.extend_from_slice(CRLF);

        if let Some(headers) = &self.headers {
            for (h_key, h_value) in headers.iter() {
                field_lines.extend(format!("{}: {}", h_key, h_value).into_bytes());
                field_lines.extend(CRLF);
            }
        };

        println!("Field Lines : {:?}", field_lines);

        // Serialize the message Body
        // Now let bring it all together
        //
        request_bytes.extend(start_line_str.into_bytes());
        request_bytes.extend(CRLF);
        request_bytes.extend(field_lines);

        // \r\n to indicate end of field lines
        request_bytes.extend(CRLF);
        request_bytes.extend(CRLF);
        request_bytes.extend_from_slice(&self.body);

        println!("Done");
        println!("{:?}", request_bytes);

        return Ok(request_bytes);
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
    while !request.is_done() {
        let idx = match request_data[read..]
            .windows(CRLF.len())
            .position(|r| r == CRLF)
        {
            Some(i) => i,
            None => break,
        };

        let end_of_line = read + idx;
        let curr_data = &request_data[read..end_of_line];

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
                let (headers, bytes_read) =
                    parse_field_lines(&request_data[read..]).map_err(|e| {
                        request.state = ParsingState::Error;
                        e
                    })?;

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
                let bytes_read =
                    parse_request_body(&request_data[read..], &mut request).map_err(|e| {
                        request.state = ParsingState::Error;
                        e
                    })?;

                read += bytes_read;
                request.state = ParsingState::Done;
            }
            ParsingState::Error => {
                println!("We are dealing with an error => ");
                break;
            }
            ParsingState::Done => {
                request.state = ParsingState::Done;
                break;
            }
        }

        // read the \r\n
    }

    Ok(request)
}

fn parse_request_line(b: &[u8]) -> Result<(RequestMethod, String, String, usize), Error> {
    // split by white space
    // result = [method SP request-target SP HTTP-version]
    // since the bytes are in UTF-8 - we have to normalize them to strings

    // Task - remove .collect() for reduce allocation
    let mut read: usize = 0;

    let x: Vec<&[u8]> = b.split(|e| *e == SP).collect();
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

    #[test]
    fn test_request_serialization() {
        let mut headers = Headers::new();
        headers.set("Host", "Localhost:8999");
        headers.set("Content-type", "application/json");

        let valid_request = Request {
            state: ParsingState::Done,
            method: Some(RequestMethod::Get),
            version: Some("1.1".to_string()),
            headers: Some(headers),
            path: Some("/api/test".to_string()),
            body: b"This is the body".to_vec(),
            param: None,
            query: None,
        };

        let raw_bytes = match valid_request.serialize() {
            Ok(s) => s,
            Err(e) => {
                panic!("Could not serialize this request because: {:?}", e)
            }
        };

        let output_str = String::from_utf8(raw_bytes).unwrap();

        println!("This is the serialized_req: {:?}", output_str);

        // let expected = b"GET /api/test HTTP/1.1\r\nHost: Localhost:8999\r\nContent-type: application/json\r\n\r\nThis is the body".to_vec();

        assert!(output_str.starts_with("GET /api/test HTTP/1.1\r\n"));

        // Assert Headers (order doesn't matter this way)
        assert!(output_str.contains("host: Localhost:8999\r\n"));
        assert!(output_str.contains("content-type: application/json\r\n"));

        // Assert Body and ending boundary
        assert!(output_str.ends_with("\r\n\r\nThis is the body"));
    }
}
