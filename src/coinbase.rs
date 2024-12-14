use crate::{
    orderbook::{OrderBook, Side as OrderSide},
    spinlock::SpinLock,
};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use serde_json::json;
use std::{fmt::Display, net::TcpStream, sync::Arc};
use tungstenite::{
    client::IntoClientRequest, connect, protocol::Message, stream::MaybeTlsStream, WebSocket,
};

type Stream = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
pub struct DecimalPair {
    pub int: u32,
    pub frac: u32,
}

impl DecimalPair {
    const ZERO: Self = Self { int: 0, frac: 0 };
}

impl Display for DecimalPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.int, self.frac)
    }
}

impl From<&DecimalPair> for f64 {
    fn from(v: &DecimalPair) -> Self {
        v.int as f64 + v.frac as f64 / 1e8
    }
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
            int: output_iter.next().unwrap(),
            frac: output_iter.next().unwrap(),
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
enum Side {
    Buy,
    Sell,
}

impl From<Side> for OrderSide {
    fn from(s: Side) -> Self {
        match s {
            Side::Buy => OrderSide::Buy,
            Side::Sell => OrderSide::Sell,
        }
    }
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

#[derive(Debug)]
pub struct CoinBaseApiClient {
    pub orderbook: Arc<SpinLock<OrderBook<DecimalPair, DecimalPair>>>,
}

impl CoinBaseApiClient {
    pub fn new() -> Self {
        let orderbook = Arc::new(SpinLock::new(OrderBook::new()));
        let new = Self { orderbook };
        new.listen_to_l2();
        new
    }

    fn listen_to_l2(&self) {
        let orderbook = Arc::clone(&self.orderbook);
        let mut stream = self.new_connection();
        self.subscribe_to_channel("level2_batch", "BTC-USD", &mut stream);
        std::thread::spawn(move || loop {
            let Message::Text(s) = stream.read().unwrap() else {
                continue;
            };
            let Ok(data): Result<MessageData, _> = serde_json::from_str(&s) else {
                continue;
            };
            match data {
                MessageData::Snapshot { bids, asks } => {
                    let mut book = orderbook.lock();
                    for b in bids {
                        book.add_order(OrderSide::Buy, b.0, b.1);
                    }
                    for a in asks {
                        book.add_order(OrderSide::Sell, a.0, a.1);
                    }
                }
                MessageData::L2Update { changes } => {
                    let mut book = orderbook.lock();
                    for (side, price, amt) in changes {
                        match amt {
                            DecimalPair::ZERO => book.remove_order(side, price),
                            _ => book.add_order(side, price, amt),
                        }
                    }
                }
            }
        });
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
}
