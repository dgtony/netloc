
use tokio;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio_io::codec::{Framed, LinesCodec};
use tokio::prelude::*;

use futures::future;

use std::net::SocketAddr;

use storage::SharedStorage;
use super::proto::{Request, Response, REASON_BAD_REQUEST};
use super::actions::process_request;

use serde_json;

/// Client processes stream of newline-delimited JSON-messages
/// responding in the client-server manner
pub struct Client<T, U> {
    stream: Framed<T, U>,
    peer_addr: SocketAddr,
    store: SharedStorage,
}


impl Client<TcpStream, LinesCodec> {
    pub fn new(s: TcpStream, store: SharedStorage) -> Self {
        let peer_addr = s.peer_addr().unwrap();
        Client {
            stream: s.framed(LinesCodec::new()),
            peer_addr,
            store,
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
                    let response = match serde_json::from_str::<Request>(&msg) {
                        Ok(request) => process_request(request, &mut self.store),
                        Err(e) => {
                            debug!(
                                "bad request from {}, error: {}, message: {}",
                                self.peer_addr,
                                e,
                                msg
                            );
                            Response::Failure { reason: REASON_BAD_REQUEST }
                        }
                    };

                    // encode and send back
                    let encoded = serde_json::to_string(&response)?;
                    self.stream.start_send(encoded)?;
                    try_ready!(self.stream.poll_complete());
                }

                None => {
                    debug!("client disconnected: {}", self.peer_addr);
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}
