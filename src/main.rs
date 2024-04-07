mod routing;
mod db;

use std::{net::SocketAddr, sync::{Arc, Mutex}};

use tokio::net::TcpListener;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut port: u16 = 5598;

    let mut args = std::env::args();
    while let Some(a) = args.next() {
        match a.as_ref() {
            "-p" => {
                if let Some(p) = args.next() {
                    port = p.parse().unwrap();
                } else {
                    println!("No port given for -p");
                }
            },
            _ => {}
        }
    }

    let db = Arc::new(Mutex::new(db::DB::new()));

    let addr = SocketAddr::from(([0,0,0,0], port));
    let listener = TcpListener::bind(addr).await?;
    let router = routing::Router::new(db);

    println!("Server started");

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);
        let r = router.clone();

        tokio::task::spawn(async move {
            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, r)
                .await {
                    println!("{:?}", e);
                }
        });
    } 
}
