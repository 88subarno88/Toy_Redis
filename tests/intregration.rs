

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use toy_redis::store::hashmap::HashMap;
use toy_redis::commands::handlers::Store; 
use toy_redis::expiry::Expiry_map;
use toy_redis::server;
use toy_redis::expiry;
use toy_redis::aof::Aof;
use toy_redis::pubsub::PubSub;
use toy_redis::store::sharded::ShardedStore;


struct TestServer {
    port: u16,
}

impl TestServer {
    fn start() -> Self {
       
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

         let store = Arc::new(ShardedStore::new());
        let expiry = Arc::new(RwLock::new(HashMap::new()));

        let aof = Aof::new("test_appendonly.aof");

        let pubsub = PubSub::new();

        expiry::strt_expiry_thread(Arc::clone(&store), Arc::clone(&expiry));

        thread::spawn(move || {
            server::run(listener, store, expiry, aof, pubsub);
        });

        thread::sleep(Duration::from_millis(50));
        TestServer { port }
    }

    fn conn(&self) -> TcpStream {
        TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap()
    }
}

fn send_recv(stream: &mut TcpStream, cmd: &[u8]) -> String {
    stream.write_all(cmd).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).unwrap();
    String::from_utf8_lossy(&buf[..n]).to_string()
}


#[test]
fn test_ping() {
    let srv = TestServer::start();
    let mut c = srv.conn();
    let resp = send_recv(&mut c, b"*1\r\n$4\r\nPING\r\n");
    assert!(resp.contains("PONG"), "expected PONG, got: {}", resp);
}

#[test]
fn test_ping_with_message() {
    let srv = TestServer::start();
    let mut c = srv.conn();
    let resp = send_recv(&mut c, b"*2\r\n$4\r\nPING\r\n$5\r\nhello\r\n");
    assert!(resp.contains("hello"), "expected hello, got: {}", resp);
}

#[test]
fn test_set_get() {
    let srv = TestServer::start();
    let mut c = srv.conn();

    let set = send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    assert!(set.contains("OK"), "SET failed: {}", set);

    let get = send_recv(&mut c, b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");
    assert!(get.contains("bar"), "GET failed: {}", get);
}

#[test]
fn test_get_missing_key() {
    let srv = TestServer::start();
    let mut c = srv.conn();
    let resp = send_recv(&mut c, b"*2\r\n$3\r\nGET\r\n$7\r\nmissing\r\n");
    assert!(resp.contains("$-1"), "expected null, got: {}", resp);
}

#[test]
fn test_del() {
    let srv = TestServer::start();
    let mut c = srv.conn();
    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n");
    let del = send_recv(&mut c, b"*2\r\n$3\r\nDEL\r\n$1\r\nk\r\n");
    assert!(del.contains(":1"), "DEL count wrong: {}", del);
    let get = send_recv(&mut c, b"*2\r\n$3\r\nGET\r\n$1\r\nk\r\n");
    assert!(get.contains("$-1"), "key should be gone: {}", get);
}

#[test]
fn test_exists() {
    let srv = TestServer::start();
    let mut c = srv.conn();
    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\nb\r\n");

    let yes = send_recv(&mut c, b"*2\r\n$6\r\nEXISTS\r\n$1\r\na\r\n");
    assert!(yes.contains(":1"), "should exist: {}", yes);

    let no = send_recv(&mut c, b"*2\r\n$6\r\nEXISTS\r\n$7\r\nnobodys\r\n");
    assert!(no.contains(":0"), "should not exist: {}", no);
}

#[test]
fn test_expire_and_ttl() {
    let srv = TestServer::start();
    let mut c = srv.conn();

    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$3\r\nttk\r\n$3\r\nval\r\n");

    let exp = send_recv(&mut c, b"*3\r\n$6\r\nEXPIRE\r\n$3\r\nttk\r\n$1\r\n2\r\n");
    assert!(exp.contains(":1"), "EXPIRE failed: {}", exp);

  
    let ttl = send_recv(&mut c, b"*2\r\n$3\r\nTTL\r\n$3\r\nttk\r\n");
    assert!(ttl.contains(":1") || ttl.contains(":2"), "TTL wrong: {}", ttl);

   
    thread::sleep(Duration::from_secs(3));
    let gone = send_recv(&mut c, b"*2\r\n$3\r\nGET\r\n$3\r\nttk\r\n");
    assert!(gone.contains("$-1"), "key should be expired: {}", gone);
}

#[test]
fn test_keys_glob() {
    let srv = TestServer::start();
    let mut c = srv.conn();

    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$1\r\nv\r\n");
    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$4\r\nkey2\r\n$1\r\nv\r\n");
    send_recv(&mut c, b"*3\r\n$3\r\nSET\r\n$5\r\nother\r\n$1\r\nv\r\n");

    let resp = send_recv(&mut c, b"*2\r\n$4\r\nKEYS\r\n$4\r\nkey*\r\n");
    assert!(resp.contains("key1"), "missing key1: {}", resp);
    assert!(resp.contains("key2"), "missing key2: {}", resp);
    assert!(!resp.contains("other"), "should not contain 'other': {}", resp);
}