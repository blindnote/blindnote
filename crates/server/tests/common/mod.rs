#![allow(dead_code, unreachable_pub)]

//! A real server on a real socket, and a small client to talk to it.
//!
//! These tests go over TCP rather than calling the router directly. That is
//! not thoroughness for its own sake: the rate limiter keys on the peer
//! address, which only exists when there is a connection, and the layers are
//! wired in an order that a direct call would not exercise.

use std::net::{Ipv4Addr, SocketAddr};

use blindnote_server::Config;
use http_body_util::BodyExt as _;
use hyper::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use hyper::{Method, Request, StatusCode};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// A running server, torn down with its data directory when dropped.
pub struct Server {
    address: SocketAddr,
    client: Client<hyper_util::client::legacy::connect::HttpConnector, String>,
    task: JoinHandle<()>,
    data_dir: TempDir,
}

/// One answer, reduced to what the tests assert on.
#[derive(Debug)]
pub struct Answer {
    /// The status line.
    pub status: StatusCode,
    /// Every response header.
    pub headers: HeaderMap,
    /// The body as text, which is what every response here is.
    pub body: String,
}

impl Answer {
    /// Parses the body as JSON.
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or_else(|error| {
            panic!("the body was not JSON ({error}): {}", self.body);
        })
    }

    /// One header as text, or `None` if it is absent.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }
}

impl Server {
    /// Starts a server with the default configuration.
    pub async fn start() -> Self {
        Self::start_with(|_| {}).await
    }

    /// The data directory this server was given.
    ///
    /// Only useful for pointing a second server at the same files, which is
    /// how the disk-floor test stores a note while there is room and then
    /// reads it back from a server that believes there is none.
    pub fn directory(&self) -> &std::path::Path {
        self.data_dir.path()
    }

    /// Starts a server, letting the caller adjust the configuration first.
    ///
    /// The listener is bound here, on port zero, and handed to
    /// [`blindnote_server::serve`]. Binding first is what lets the test learn
    /// the port the kernel chose.
    pub async fn start_with(adjust: impl FnOnce(&mut Config)) -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");

        let mut config = Config {
            data_dir: directory.path().to_path_buf(),
            ..Config::default()
        };
        adjust(&mut config);

        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .expect("a listener on an ephemeral port");
        let address = listener.local_addr().expect("the chosen address");

        let task = tokio::spawn(async move {
            blindnote_server::serve(config, listener)
                .await
                .expect("the server runs");
        });

        Self {
            address,
            client: Client::builder(TokioExecutor::new()).build_http(),
            task,
            data_dir: directory,
        }
    }

    /// Sends a request with no body.
    pub async fn send(&self, method: Method, path: &str) -> Answer {
        self.request(method, path, None).await
    }

    /// Sends a `POST` with a JSON body.
    pub async fn post_json(&self, path: &str, body: &str) -> Answer {
        self.request(Method::POST, path, Some(body.to_owned()))
            .await
    }

    /// Creates a note and returns its id, failing the test if it cannot.
    pub async fn create(&self, ciphertext: &str) -> String {
        let answer = self
            .post_json("/api/notes", &format!(r#"{{"ciphertext":"{ciphertext}"}}"#))
            .await;

        assert_eq!(
            answer.status,
            StatusCode::OK,
            "create failed: {}",
            answer.body
        );
        answer.json()["id"]
            .as_str()
            .expect("an id in the response")
            .to_owned()
    }

    /// Sends a request carrying one extra header.
    pub async fn send_with_header(
        &self,
        method: Method,
        path: &str,
        name: &'static str,
        value: &str,
    ) -> Answer {
        self.request_with(method, path, None, Some((name, value.to_owned())))
            .await
    }

    async fn request(&self, method: Method, path: &str, body: Option<String>) -> Answer {
        self.request_with(method, path, body, None).await
    }

    async fn request_with(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
        extra: Option<(&'static str, String)>,
    ) -> Answer {
        let uri = format!("http://{}{path}", self.address);
        let mut builder = Request::builder().method(method).uri(uri);

        if body.is_some() {
            builder = builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        if let Some((name, value)) = extra {
            builder = builder.header(name, value);
        }

        let request = builder
            .body(body.unwrap_or_default())
            .expect("a valid request");
        let response = self.client.request(request).await.expect("a response");

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("a complete body")
            .to_bytes();

        Answer {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A 16-byte ciphertext, unpadded base64url: the smallest a note can be.
///
/// AES-GCM's tag is 16 bytes and it adds no padding, so this is what the
/// browser sends for an empty note.
pub const SMALLEST_CIPHERTEXT: &str = "AAAAAAAAAAAAAAAAAAAAAA";

/// A plausible note-sized ciphertext.
pub fn ciphertext_of(bytes: usize) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(vec![0xAB; bytes])
}
