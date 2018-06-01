/// Client interface
///
/// Implements request/response JSON-based protocol server,
/// that could be used to obtain current network coordinates.

use log;
use bytes::{BytesMut, Bytes, BufMut};

use tokio;
use tokio::runtime::current_thread::Runtime;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio_io::codec::{Framed, LinesCodec};
use tokio::prelude::*;

use futures::future::{self, Either};

use storage;

use std::net::SocketAddr;

mod actions;
mod client;
mod proto;

use self::client::Client;

// TODO:
// use tokio
// executor: CurrentThread
// codec: LineCodec (new line delimited)
//
// Processing task:
// - read line
// - try to deserialize it into command structure (serde_json)
// - acquire storage lock
// - get data from storage and release
// - transform data, make calculations (if any) and form reply
// - serialize reply structure
// - send reply back


fn process_stream(stream: TcpStream, store: storage::SharedStorage) {
    tokio::spawn(Client::new(stream).map_err(|e| {
        error!("interface client error: {}", e)
    }));
}


// todo change signature:
// pub fn run_interface(addr: SocketAddr, store: storage::SharedStorage) -> io::Result<()> {
pub fn run_interface(store: storage::SharedStorage) -> io::Result<()> {

    let addr = "127.0.0.1:5555".parse().unwrap();

    //let listener = TcpListener::bind(&addr)?;

    debug!("interface server started at {}", addr);

    let server = TcpListener::bind(&addr)?
        .incoming()
        .for_each(move |stream| {
            let peer_addr = stream.peer_addr()?;

            // todo refine?
            debug!(
                "interface client connected: {}:{}",
                peer_addr.ip(),
                peer_addr.port()
            );

            process_stream(stream, store.clone());
            Ok(())
        })
        .map_err(|e| println!("accept error: {}", e));

    // single threaded runtime
    let mut r = Runtime::new().unwrap();
    let _ = r.spawn(server).run();

    Ok(())
}
