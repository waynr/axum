//! Example websocket server.
//!
//! Run with
//!
//! ```
//! RUST_LOG=tower_http=debug,key_value_store=trace \
//!     cargo run \
//!     --features ws \
//!     --example websocket
//! ```

use http::StatusCode;
use hyper::Server;
use std::net::SocketAddr;
use tower::make::Shared;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tower_web::{
    prelude::*,
    routing::nest,
    service::ServiceExt,
    ws::{ws, Message, WebSocket},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // build our application with some routes
    let app = nest(
        "/",
        ServeDir::new("examples/websocket")
            .append_index_html_on_directories(true)
            .handle_error(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    )
    // routes are matched from bottom to top, so we have to put `nest` at the
    // top since it matches all routes
    .route("/ws", ws(handle_socket))
    // logging so we can see whats going on
    .layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let server = Server::bind(&addr).serve(Shared::new(app));
    server.await.unwrap();
}

async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        let msg = msg.unwrap();
        println!("Client says: {:?}", msg);
    }

    loop {
        socket.send(Message::text("Hi!")).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}