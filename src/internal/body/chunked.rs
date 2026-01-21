use std::{
    io::{Error, ErrorKind},
    str,
};
const CRLF: &[u8; 2] = b"\r\n";
const MAX_LENGTH: usize = 1024;

pub fn parse_chunked_message(msg: &[u8], body: &mut Vec<u8>) -> Result<usize, Error> {
    enum ChunkState {
        GetSize,
        GetData,
    }

    let mut state: ChunkState = ChunkState::GetSize;
    let mut size: usize = 0;
    let mut read: usize = 0;

    if msg.len() >= MAX_LENGTH {
        return Err(Error::new(ErrorKind::InvalidInput, "Exceeded max length"));
    }

    loop {
        let chunk_idx = match msg[read..].windows(2).position(|b| b == CRLF) {
            Some(c) => c,
            None => return Ok(read),
        };

        let chunk = &msg[read..read + chunk_idx];

        match state {
            ChunkState::GetSize => {
                let chunk_str = str::from_utf8(chunk)
                    .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid utf-8 in chunks"))?;

                let chunk_usize = usize::from_str_radix(chunk_str, 16).map_err(|_| {
                    Error::new(ErrorKind::InvalidInput, "Invalid chunk size number")
                })?;

                if chunk_usize == 0 {
                    return Ok(read + chunk_idx + 4);
                }
                size = chunk_usize;

                state = ChunkState::GetData;
                read += chunk_idx + 2;
            }

            ChunkState::GetData => {
                let required_byte = size + 2;

                if msg[read..].len() < required_byte {
                    return Ok(read);
                }

                //  Unneccessary check;
                // if msg[read..required_byte + read].len() != size {
                //     println!("This is it");
                //     return Err(Error::new(ErrorKind::InvalidInput, "Invalid chunk size"));
                // }

                let data = &msg[read..read + required_byte];

                body.extend_from_slice(data);

                if &msg[read + size..read + required_byte] != b"\r\n" {
                    return Err(Error::new(ErrorKind::InvalidData, "Chunk missing CRLF"));
                }

                state = ChunkState::GetSize;
                read += required_byte
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk_body() {
        let mut body: Vec<u8> = Vec::new();
        let mut input: &[u8] = b"6\r\nHello \r\n5\r\nWorld\r\n0\r\n\r\n";

        println!("Ox");
        let result = parse_chunked_message(input, &mut body).unwrap();

        let hello = b"Hello";
        assert_eq!(result, 26);
        let result_slice = &body[0..5];
        assert_eq!(result_slice, hello);
        println!("O");

        input = b"6\r\nHello \r\n%\r\nWorld\r\n0\r\n\r\n";
        body = Vec::new();
        let result_err = parse_chunked_message(input, &mut body).unwrap_err();

        println!("result_err = {result_err:?}");

        assert_eq!(result_err.kind(), ErrorKind::InvalidInput);
        assert_eq!(result_err.to_string().as_str(), "Invalid chunk size number");
    }
}
