use std::{
    collections::HashMap,
    io::{self, Write},
};

//RESPONSE SCHEMATICS -> [start-line]CRLF[headers]CRLF[message-body]
pub struct Response {
    pub protocol: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub entity: Option<Vec<u8>>,
}

impl Response {
    pub fn new(status_code: u16, status_text: &str, entity: Option<Vec<u8>>) -> Response {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Server".to_string(), "X-B-O-X".to_string());
        headers.insert("Cmnnection".to_string(), "close".to_string());

        if let Some(ref e) = entity {
            headers.insert("Content-Length".to_string(), e.len().to_string());
        } else {
            headers.insert("Content-Length".to_string(), 0.to_string());
        };

        Response {
            protocol: "HTTP/1.1".to_string(),
            status_code,
            status_text: status_text.to_string(),
            headers,
            entity,
        }
    }

    pub fn send(&self, stream: &mut impl Write) -> io::Result<()> {
        // append the startling
        write!(
            stream,
            "{} {} {}\r\n",
            self.protocol, self.status_code, self.status_text
        )?;
        // append the hash_map
        for (key, value) in &self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }
        // append the CRLF after the headers
        write!(stream, "\r\n")?;

        //append the body if it exist
        if let Some(ref body) = self.entity {
            stream.write_all(body)?;
        };

        //send the data in buffer to stream
        stream.flush()
    }

    pub fn ok(stream: &mut impl Write, message: Option<&[u8]>) -> io::Result<()> {
        let status_code: u16 = 200;
        let status_text: &str = "OK";
        let entity: Option<Vec<u8>> = match message {
            Some(s) => Some(s.to_vec()),
            None => Some(b"OK".to_vec()),
        };

        let response = Response::new(status_code, status_text, entity);

        response.send(stream)
    }

    pub fn not_found(stream: &mut impl Write, message: Option<&[u8]>) -> io::Result<()> {
        let status_code: u16 = 404;
        let status_text: &str = "NOT FOUND";

        let entity: Option<Vec<u8>> = match message {
            Some(s) => Some(s.to_vec()),
            None => Some(b"Resource Not Found".to_vec()),
        };

        let response = Response::new(status_code, status_text, entity);

        response.send(stream)
    }

    pub fn bad_request(stream: &mut impl Write, message: Option<&[u8]>) -> io::Result<()> {
        let status_code: u16 = 400;
        let status_text: &str = "BAD REQUEST";
        let entity: Option<Vec<u8>> = match message {
            Some(s) => Some(s.to_vec()),
            None => Some(b"Bad Request".to_vec()),
        };

        let response = Response::new(status_code, status_text, entity);

        response.send(stream)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ok_response() {
        let mut mock_socket = Vec::new();

        Response::ok(&mut mock_socket, Some(b"Test is OK.")).unwrap();

        let output = String::from_utf8_lossy(&mock_socket);

        assert!(output.contains("Test is OK."));
        assert!(output.contains("HTTP/1.1"));
        assert!(output.contains("200 OK"));
    }

    #[test]
    fn test_not_found() {
        let mut mock_socket = Vec::new();

        Response::not_found(&mut mock_socket, Some(b"Test is Not Found.")).unwrap();

        let output = String::from_utf8_lossy(&mock_socket);

        assert!(output.contains("Test is Not Found."));
        assert!(output.contains("HTTP/1.1"));
        assert!(output.contains("404 NOT FOUND"));
    }

    #[test]
    fn test_bad_request() {
        let mut mock_socket = Vec::new();

        Response::bad_request(&mut mock_socket, Some(b"Test is Bad Request.")).unwrap();

        let output = String::from_utf8_lossy(&mock_socket);
        // println!("{output}");

        assert!(output.contains("Test is Bad Request."));
        assert!(output.contains("HTTP/1.1"));
        assert!(output.contains("400 BAD REQUEST"));
    }
}
