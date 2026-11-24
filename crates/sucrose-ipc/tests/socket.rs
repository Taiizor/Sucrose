//! The channel over a real local socket: bind, connect, single instance and survival of bad peers.
//! Every test uses its own endpoint name so the tests can run in parallel.

use std::{
    sync::atomic::{AtomicU32, Ordering},
    sync::mpsc,
    thread,
    time::Duration,
};

use sucrose_ipc::{
    IpcError, Listener, connect,
    protocol::{PROTOCOL_VERSION, ToRenderer, ToService},
};

fn unique_name() -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!(
        "Sucrose.Test.{}.{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn hello() -> ToService {
    ToService::Hello {
        protocol_version: PROTOCOL_VERSION,
        pid: std::process::id(),
        slot: 0,
    }
}

#[test]
fn a_second_listener_on_the_same_name_is_already_running() {
    let name = unique_name();
    let _first = Listener::bind(&name, None).expect("first bind");

    let second = Listener::bind(&name, None);

    assert!(matches!(second, Err(IpcError::AlreadyRunning)), "{second:?}");
}

#[test]
fn hello_and_close_round_trip_over_a_real_socket() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let server = thread::spawn(move || {
        let mut connection = listener.accept().expect("accept");
        let received: ToService = connection.recv().expect("recv Hello");
        connection.send(&ToRenderer::Close).expect("send Close");
        received
    });

    let mut client = connect(&name).expect("connect");
    client.send(&hello()).expect("send Hello");
    let reply: ToRenderer = client.recv().expect("recv Close");

    assert_eq!(reply, ToRenderer::Close);
    assert_eq!(server.join().expect("server thread"), hello());
}

#[test]
fn a_peer_that_sends_garbage_does_not_stop_the_listener() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let (accepted, bad_accepted) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut bad = listener.accept().expect("accept the bad peer");
        accepted.send(()).expect("tell the client");
        let bad_result = bad.recv::<ToService>();
        drop(bad);
        let mut good = listener.accept().expect("accept the next peer");
        let good_result = good.recv::<ToService>();
        (bad_result, good_result)
    });

    // A valid frame whose payload is not a ToService: postcard reads variant index 4294967295.
    let mut bad = connect(&name).expect("connect bad peer");
    bad.send(&u32::MAX).expect("send garbage");
    // A named pipe client that is gone before accept() runs leaves nothing to accept, so wait for the server.
    bad_accepted.recv().expect("the server accepted the bad peer");
    drop(bad);
    let mut good = connect(&name).expect("connect the next peer");
    good.send(&hello()).expect("send Hello");

    let (bad_result, good_result) = server.join().expect("server thread");
    assert!(matches!(bad_result, Err(IpcError::Decode(_))), "{bad_result:?}");
    assert_eq!(good_result.expect("Hello from the next peer"), hello());
}

#[test]
fn a_closed_peer_reads_as_closed() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let (accepted, server_accepted) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut connection = listener.accept().expect("accept");
        accepted.send(()).expect("tell the client");
        connection.recv::<ToService>()
    });

    let client = connect(&name).expect("connect");
    // A named pipe client that is gone before accept() runs leaves nothing to accept, so wait for the server.
    server_accepted.recv().expect("the server accepted the client");
    drop(client);

    let result = server.join().expect("server thread");
    assert!(matches!(result, Err(IpcError::Closed)), "{result:?}");
}

#[test]
fn a_second_handle_can_send_while_the_first_waits_for_a_message() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let server = thread::spawn(move || {
        let connection = listener.accept().expect("accept");
        let mut reader = connection.try_clone().expect("clone");
        let mut writer = connection;
        let waiting = thread::spawn(move || reader.recv::<ToService>());
        // The reader thread is (about to be) blocked in recv; sending on the other handle must not wait for it.
        writer.send(&ToRenderer::Close).expect("send while the reader waits");
        waiting.join().expect("reader thread")
    });

    let mut client = connect(&name).expect("connect");
    let reply: ToRenderer = client.recv().expect("recv Close");
    client.send(&hello()).expect("send Hello");

    assert_eq!(reply, ToRenderer::Close);
    assert_eq!(server.join().expect("server thread").expect("Hello"), hello());
}

#[cfg(windows)]
#[test]
fn a_disconnected_connection_wakes_a_blocked_reader() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let accepting = thread::spawn(move || listener.accept().expect("accept"));
    let mut client = connect(&name).expect("connect");
    let server = accepting.join().expect("accept thread");
    let closer = server.try_clone().expect("clone");
    let (woke, reader_woke) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut server = server;
        let result = server.recv::<ToService>();
        woke.send(result.is_err()).expect("tell the test");
    });
    // The reader is (about to be) blocked in recv; if the disconnect wins the race recv fails at once, which the
    // test accepts too.
    thread::sleep(Duration::from_millis(100));

    closer.disconnect().expect("disconnect");

    let reader_failed = reader_woke
        .recv_timeout(Duration::from_secs(2))
        .expect("the blocked reader returned within 2 s");
    assert!(reader_failed, "the reader must see an error, not a message");
    reader.join().expect("reader thread");
    let result = client.recv::<ToRenderer>();
    assert!(
        matches!(result, Err(IpcError::Closed | IpcError::Io(_))),
        "the client must see the connection end: {result:?}"
    );
}

#[cfg(windows)]
#[test]
fn disconnecting_twice_or_after_the_peer_left_is_harmless() {
    let name = unique_name();
    let listener = Listener::bind(&name, None).expect("bind");
    let accepting = thread::spawn(move || listener.accept().expect("accept"));
    let client = connect(&name).expect("connect");
    let server = accepting.join().expect("accept thread");
    drop(client);

    let first = server.disconnect();
    let second = server.disconnect();

    // Either call may report an error (nothing is left to disconnect); neither may panic.
    let _ = (first, second);
}

#[cfg(windows)]
#[test]
fn a_listener_restricted_to_the_current_sid_still_serves_its_owner() {
    let sid = sucrose_platform::current().user_key().expect("user key");
    let name = unique_name();
    let listener = Listener::bind(&name, Some(&sid)).expect("bind restricted to the current user");
    let server = thread::spawn(move || listener.accept().expect("accept").recv::<ToService>());

    let mut client = connect(&name).expect("the owner can connect");
    client.send(&hello()).expect("send Hello");

    assert_eq!(server.join().expect("server thread").expect("Hello"), hello());
}

#[cfg(windows)]
#[test]
fn an_owner_that_is_not_a_sid_is_refused() {
    let name = unique_name();

    let result = Listener::bind(&name, Some("S-1-5-21)(A;;GA;;;WD"));

    assert!(matches!(result, Err(IpcError::Io(_))), "{result:?}");
}
