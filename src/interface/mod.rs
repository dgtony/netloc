/// Client interface
///
/// Implements request/response JSON-based protocol server,
/// that could be used to obtain current network coordinates.

use log;
use bytes::{BytesMut, Bytes, BufMut};

use tokio;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

use tokio_io::codec::{Framed, LinesCodec};
use futures::future::{self, Either};

use std::net::SocketAddr;

use storage;
use self::client::Client;

mod actions;
mod client;
mod proto;


fn process_stream(stream: TcpStream, store: storage::SharedStorage) {
    tokio::spawn(Client::new(stream, store).map_err(|e| {
        error!("interface client error: {}", e)
    }));
}


pub fn run_server(addr: SocketAddr, store: storage::SharedStorage) -> io::Result<()> {
    debug!("interface server started at {}", addr);
    let server = TcpListener::bind(&addr)?
        .incoming()
        .for_each(move |stream| {
            let peer_addr = stream.peer_addr()?;
            info!("client connected: {}", peer_addr);

            process_stream(stream, store.clone());
            Ok(())
        })
        .map_err(|e| error!("accept connection: {}", e));

    // single threaded runtime
    let mut r = Runtime::new().unwrap();
    let _ = r.spawn(server).run();

    Ok(())
}
