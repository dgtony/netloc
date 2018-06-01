
use tokio;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio_io::codec::{Framed, LinesCodec};
use tokio::prelude::*;

use futures::future;

use std::net::SocketAddr;

use super::proto::*;
use super::actions::process_request;

use serde_json;

/// Client processes stream of newline-delimited JSON-messages
/// responding in the client-server manner
pub struct Client<T, U> {
    stream: Framed<T, U>,
    peer_addr: SocketAddr,
}


impl Client<TcpStream, LinesCodec> {
    pub fn new(s: TcpStream) -> Self {
        let peer_addr = s.peer_addr().unwrap();
        Client {
            stream: s.framed(LinesCodec::new()),
            peer_addr,
        }
    }
}

impl Future for Client<TcpStream, LinesCodec> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // process until closed
        loop {
            match try_ready!(self.stream.poll()) {
                Some(msg) => {

                    // todo remove
                    debug!("client {} | read line from socket: {}", self.peer_addr, msg);

                    // todo decode & process request
                    let response = match serde_json::from_str::<Request>(&msg) {
                        Ok(request) => {
                            // todo remove
                            println!("request parsed: {:?}", request);

                            process_request(request)
                        }

                        Err(e) => {
                            println!("get bad request: {} | reason: {}", msg, e);
                            Response::Failure {
                                reason: Some(REASON_BAD_MESSAGE),
                            }
                        }
                    };

                    // encode and send back
                    let encoded = serde_json::to_string(&response)?;
                    self.stream.start_send(encoded)?;
                    try_ready!(self.stream.poll_complete());
                }

                None => {
                    debug!("client closed: {}", self.peer_addr);
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}
