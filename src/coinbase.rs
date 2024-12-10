//#![allow(dead_code)]
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use serde_json::json;
use std::{collections::BTreeMap, io::Write, marker::ConstParamTy, net::TcpStream};
use tungstenite::{
    client::IntoClientRequest, connect, protocol::Message, stream::MaybeTlsStream, WebSocket,
};

type Stream = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
struct OrderBook {
    bids: BTreeMap<OrderBookKey<{ Side::Buy }>, f32>,
    asks: BTreeMap<OrderBookKey<{ Side::Sell }>, f32>,
    max_depth: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct OrderBookKey<const S: Side>(DecimalPair);

impl<const S: Side> Ord for OrderBookKey<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match S {
            Side::Buy => other.0.cmp(&self.0),
            Side::Sell => self.0.cmp(&other.0),
        }
    }
}

impl<const S: Side> PartialOrd for OrderBookKey<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
struct DecimalPair {
    integer: u32,
    fraction: u32,
}

struct DecimalPairVisitor;

impl Visitor<'_> for DecimalPairVisitor {
    type Value = DecimalPair;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A decimal number like 123.234")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut output_iter = value.split('.').map(|x| str::parse(x).unwrap());
        Ok(DecimalPair {
            integer: output_iter.next().unwrap(),
            fraction: output_iter.next().unwrap(),
        })
    }
}

impl<'de> Deserialize<'de> for DecimalPair {
    fn deserialize<D>(deserializer: D) -> Result<DecimalPair, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalPairVisitor)
    }
}

#[derive(Deserialize, Debug, ConstParamTy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Side {
    Buy,
    Sell,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum MessageData {
    Snapshot {
        bids: Vec<(DecimalPair, DecimalPair)>,
        asks: Vec<(DecimalPair, DecimalPair)>,
    },
    L2Update {
        changes: Vec<(Side, DecimalPair, DecimalPair)>,
    },
}

pub struct CoinBaseApiClient {}

impl CoinBaseApiClient {
    pub fn new() -> Self {
        Self {}
    }
    fn new_connection(&self) -> Stream {
        const USE_PROD: bool = true;
        let url = if USE_PROD {
            "wss://ws-feed.exchange.coinbase.com"
        } else {
            "wss://ws-feed-public.sandbox.exchange.coinbase.com"
        };
        let request = url.into_client_request().unwrap();
        let (stream, _response) = connect(request).unwrap();
        stream
    }

    fn subscribe_to_channel(&self, channel: &str, product_id: &str, stream: &mut Stream) {
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
        stream.send(subscribe_msg).expect("send message failed");
    }

    fn listen_to_channel(&self, stream: &mut Stream) {
        let mut stdout = std::io::stdout().lock();
        loop {
            let Message::Text(s) = stream.read().unwrap() else {
                continue;
            };
            let Ok(data): Result<MessageData, _> = serde_json::from_str(&s) else {
                continue;
            };
            stdout.write_all(format!("{data:#?}").as_bytes()).unwrap();
        }
    }

    pub fn connect_to_api(&self) {
        let mut stream = self.new_connection();
        self.subscribe_to_channel("level2_batch", "BTC-USD", &mut stream);
        self.listen_to_channel(&mut stream);
    }
}
