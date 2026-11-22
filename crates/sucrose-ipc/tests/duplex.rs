//! The service reads a renderer's connection on one thread and must still be able to write to it (Close) from
//! another thread while that read is blocked.

use std::{sync::mpsc, thread, time::Duration};

use sucrose_ipc::{
    Listener, connect,
    protocol::{ToRenderer, ToService},
};

#[test]
fn a_blocked_read_does_not_block_a_write_on_a_clone_of_the_same_end() {
    let name = format!("Sucrose.Test.Duplex.{}", std::process::id());
    let listener = Listener::bind(&name, None).expect("bind");
    let client_name = name.clone();
    let client = thread::spawn(move || {
        let mut connection = connect(&client_name).expect("connect");
        connection
            .send(&ToService::Hello {
                protocol_version: 1,
                pid: 1,
                slot: 0,
            })
            .expect("hello");
        connection.recv::<ToRenderer>().expect("the close message")
    });

    let mut server = listener.accept().expect("accept");
    let _hello: ToService = server.recv().expect("hello");
    let mut writer = server.try_clone().expect("clone");
    // The reader blocks in recv on the server end, like the service's reader thread does.
    let (done_sender, done_receiver) = mpsc::channel();
    let _reader = thread::spawn(move || {
        let result = server.recv::<ToService>();
        let _ = done_sender.send(result.is_err());
    });
    thread::sleep(Duration::from_millis(300));

    let (sent_sender, sent_receiver) = mpsc::channel();
    let _writer = thread::spawn(move || {
        let _ = sent_sender.send(writer.send(&ToRenderer::Close));
    });

    let sent = sent_receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("the write must not wait for the blocked read");
    sent.expect("send Close");
    assert_eq!(client.join().expect("client thread"), ToRenderer::Close);
    // The client has gone: the blocked read ends with an error rather than hanging.
    assert!(
        done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("the read ends")
    );
}
