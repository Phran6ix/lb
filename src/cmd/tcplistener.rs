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

    Ok(received)
}

pub fn listen_for_http() -> Result<(), Error> {
    let socket_url = "127.0.0.1:8080";
    let listener = net::TcpListener::bind(&socket_url).expect("Could not bind to port.!!!");

    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(mut stream) => {
                println!("====================");

                let request_data = match process_request_data(&mut stream) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("An error occured when writing bytes to stream, {e}");
                        break;
                    }
                };

                if let Err(e) = request::parse(&request_data) {
                    match e.kind() {
                        ErrorKind::InvalidData | ErrorKind::InvalidInput => {
                            let response =
                                Response::new(400, "Bad Request", Some(e.to_string().into_bytes()));
                            response.send(&mut stream).ok();
                            break;
                        }
                        ErrorKind::Unsupported => {
                            let response =
                                Response::new(415, "Unsupported", Some(e.to_string().into_bytes()));
                            response.send(&mut stream).ok();
                            break;
                        }
                        _ => {
                            let response = Response::new(500, "Internal Server Error", None);
                            response.send(&mut stream).ok();
                            break;
                        }
                    }
                };

                let _ = Response::ok(&mut stream, Some(b"Hello from Rust World\n"));
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
