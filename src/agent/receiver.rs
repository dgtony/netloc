/// Process all incoming UDP packets.
///
/// Possible messages:
/// - Bootstrap response;
/// - Location request (foreign);
/// - Location response (for the local request).

use storage;

//fn recv_udp(sock: UdpSocket) {
//    let mut buff = Vec::new();
//    if let Ok((msg_len, sender)) = sock.recv_from(&mut buff) {
//        let node_addr = sender.ip();
//        let node_port = sender.port();
//
//        // todo process
//    }
//}
