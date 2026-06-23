use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    net::{self, TcpStream},
};

use lb::{Request, Response, Router, internal::request};
// use crate::{request, response::Response};

fn consume_request_data(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    // read the stream into a buffer
    let mut reader = BufReader::new(stream);

    // read the data  into a byte array
    let received: Vec<u8> = reader.fill_buf()?.to_vec();

    // consume
    reader.consume(received.len());

    Ok(received)
}

pub fn listen_for_http(router: &mut Router) -> Result<(), Error> {
    let socket_url = "127.0.0.1:8080";
    let listener = net::TcpListener::bind(&socket_url).expect("Could not bind to port.!!!");

    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(mut stream) => {
                println!("====================");

                // Rewrite the consume_request_data to set a fix buffer for the tcp stream -
                // prioritize the headers to know the Content-Type in case the request has a
                // Body
                //
                let stream_data = match consume_request_data(&mut stream) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("An error occured when writing bytes to stream, {e}");
                        continue;
                    }
                };

                let mut request_data: Request = match request::parse(&stream_data) {
                    Ok(data) => data,
                    Err(e) => match e.kind() {
                        ErrorKind::InvalidData | ErrorKind::InvalidInput => {
                            let response =
                                Response::new(400, "Bad Request", Some(e.to_string().into_bytes()));
                            if let Err(e) = response.send(&mut stream) {
                                eprintln!("Failed to send response to stream because {}", e);
                            };
                            continue;
                        }
                        ErrorKind::Unsupported => {
                            let response =
                                Response::new(415, "Unsupported", Some(e.to_string().into_bytes()));
                            if let Err(e) = response.send(&mut stream) {
                                eprintln!("Failed to send response to stream because {}", e);
                            };
                            continue;
                        }
                        _ => {
                            let response = Response::new(500, "Internal Server Error", None);
                            if let Err(e) = response.send(&mut stream) {
                                eprintln!("Failed to send response to stream because {}", e);
                            };
                            continue;
                        }
                    },
                };

                let Some(path) = &request_data.path.take() else {
                    println!("No path >????");
                    let response = Response::new(400, "Bad Request", None);
                    if let Err(e) = response.send(&mut stream) {
                        eprintln!("Failed to send response to stream because {}", e);
                    };
                    continue;
                };

                let Some(method) = &request_data.method.take() else {
                    println!("No Method>????");
                    let response = Response::new(400, "Bad Request", None);
                    if let Err(e) = response.send(&mut stream) {
                        eprintln!("Failed to send response to stream because {}", e);
                    };
                    continue;
                };
                // A cleaner and better version is the to use Option::take()
                // let path_string = path.to_string();
                // let method = *method;
                //
                // router.resolve_path(&mut request_data, &path_string,  &method);

                let router_attached_method =
                    match router.resolve_path(&mut request_data, &path, &method) {
                        Ok(handler) => handler,
                        Err(e) => {
                            println!("Wtf happened");

                            let response =
                                Response::new(400, e.as_str(), Some(e.as_bytes().to_vec()));
                            println!("response => {:?}", response);

                            if let Err(e) = response.send(&mut stream) {
                                eprintln!("Failed to send response to stream because {}", e);
                            };
                            continue;
                        }
                    };

                let response = router_attached_method(&request_data);

                if let Err(e) = response.send(&mut stream) {
                    eprintln!("Failed to send response to stream because {}", e);
                };

                // let _ = Response::ok(&mut stream, Some(b"Hello from Rust World\n"));
                continue;
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
