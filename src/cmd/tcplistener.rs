use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    net::{self, TcpStream},
};

use crate::internal::{request, response::Response};

fn process_request_data(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    // read the stream into a buffer
    let mut reader = BufReader::new(stream);

    // read the data  into a byte array
    let received: Vec<u8> = reader.fill_buf()?.to_vec();

    reader.consume(received.len());

    println!("received VALUE => {received:?}");
    Ok(received)
}

pub fn listen_for_http() -> Result<(), Error> {
    let socket_url = "127.0.0.1:8080";
    let listener = net::TcpListener::bind(&socket_url)?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut data) => {
                println!("====================");
                println!("stream data received ");

                let request_data = process_request_data(&mut data)?;

                let _ = request::parse(&request_data);
                let _ = Response::ok(&mut data, None);
                println!("Stream done processing");
            }
            Err(e) => {
                println!("Error occured on stream: {}", e);
                return Err(Error::new(
                    ErrorKind::NetworkUnreachable,
                    "Something went wrong",
                ));
            }
        }
    }

    Ok(())
}
