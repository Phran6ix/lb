use std::io::{Error, ErrorKind};
const MAX_LENGTH: usize = 1024;

pub fn parse_fixed_message(msg: &[u8], size: usize, body: &mut Vec<u8>) -> Result<usize, Error> {
    if msg.len() >= MAX_LENGTH {
        return Err(Error::new(ErrorKind::InvalidInput, "Exceeded max length"));
    }

    if msg.len() < size {
        return Err(Error::new(
            ErrorKind::WouldBlock,
            "Incomplete message: waiting for data",
        ));
    }

    let request_body = &msg[0..size];

    if msg.len() < size {
        return Err(Error::new(ErrorKind::InvalidData, "Waiting for Data.."));
    };

    body.extend_from_slice(request_body);

    Ok(body.len())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_fixed_message() {
        let mut body: Vec<u8> = Vec::new();
        let mut input: &[u8] = b"This is a valid input, okay";
        let mut content_length: usize = 27;

        let mut result = parse_fixed_message(input, content_length, &mut body).unwrap();

        assert_eq!(result, 27);
        assert_eq!(&body[0..4], b"This");
        assert_eq!(&body[body.len() - 4..], b"okay");

        input = b"This is a cut, exceed content length";
        content_length = 13;

        body = Vec::new();
        result = parse_fixed_message(input, content_length, &mut body).unwrap();

        assert_eq!(result, 13);
        assert_eq!(&body[body.len() - 4..], b" cut");

        println!("This one");
        input = b"shorter one";
        content_length = 13;
        let result = parse_fixed_message(input, content_length, &mut body).unwrap_err();

        assert_eq!(result.kind(), ErrorKind::WouldBlock);
        assert_eq!(
            result.to_string().as_str(),
            "Incomplete message: waiting for data"
        );
    }
}
