use market::Market;
use player::PlayerActor;
use tokio::net::TcpListener;

pub mod market;
pub mod order_book;
pub mod player;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr).await.expect("Unable to listen");
    println!("Listenning on {addr}");

    let mut market = Market::new();
    let tx_martket = market.get_tx();

    tokio::spawn(async move {
        println!("Starting market actor");
        market.process().await;
    });

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("No peer address");
        println!("Connection from {peer}");

        let tx = tx_martket.clone();
        tokio::spawn(async move {
            let _ = PlayerActor::start(stream, tx).await;
        });
    }
}
