use std::{cmp::Reverse, collections::BTreeMap, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct OrderBook<K: Ord, V> {
    bids: BTreeMap<Reverse<K>, V>,
    asks: BTreeMap<K, V>,
}

impl<K: Ord, V> OrderBook<K, V> {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
    pub fn add_order(&mut self, side: Side, key: K, val: V) {
        match side {
            Side::Buy => {
                self.bids.insert(Reverse(key), val);
            }
            Side::Sell => {
                self.asks.insert(key, val);
            }
        }
    }

    pub fn remove_order(&mut self, side: Side, key: K) {
        match side {
            Side::Buy => {
                self.bids.remove(&Reverse(key));
            }
            Side::Sell => {
                self.asks.remove(&key);
            }
        }
    }

    pub fn clear_book(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }
}

impl<K: Ord + Display, V: Display> Display for OrderBook<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write! {f, "<"}?;
        for (key, value) in self.bids.iter().take(2).rev() {
            let key = &key.0;
            write! {f, "[{key}:{value}]"}?
        }
        write! {f, "|"}?;
        for (key, value) in self.asks.iter().take(2) {
            write! {f, "[{key}: {value}]"}?
        }
        write! {f, ">"}
    }
}
