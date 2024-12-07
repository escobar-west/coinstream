use std::{collections::BTreeMap, sync::mpsc};

use futures::{join, SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<String, f32>,
    pub asks: BTreeMap<String, f32>,
}

struct CoinBaseApiClient {}

impl CoinBaseApiClient {
    async fn subscribe(&self, channel: &str, product_id: &str, tx: &mpsc::Sender<String>) {
        let request = "wss://ws-feed-public.sandbox.exchange.coinbase.com"
            //let request = "wss://ws-feed.exchange.coinbase.com"
            .into_client_request()
            .unwrap();
        let (mut stream, _response) = connect_async(request).await.unwrap();
        let subscribe_msg = Message::text(
            json!({
                "type": "subscribe",
                "product_ids": [
                    product_id
                ],
                "channels": [channel]
            })
            .to_string(),
        );
        stream
            .send(subscribe_msg)
            .await
            .expect("send message failed");
        while let Some(Ok(Message::Text(s))) = stream.next().await {
            tx.send(s).unwrap();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn connect_to_api(tx: mpsc::Sender<String>) {
    let cb_client = CoinBaseApiClient {};
    let l2 = cb_client.subscribe("level2_batch", "BTC-USD", &tx);
    l2.await;
}

fn main() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || connect_to_api(tx));
    loop {
        let msg = rx.recv().unwrap();
        println!("{msg}");
    }
}
