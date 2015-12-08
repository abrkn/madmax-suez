extern crate bincode;
extern crate rustc_serialize;

use std::cmp;
use std::collections::HashMap;

pub type OrderId = u64;
pub type OrderSize = u64;
pub type OrderPrice = u64;
pub type MarketId = u32;
pub type AssetId = u32;
pub type UserId = u32;

#[derive(RustcEncodable, RustcDecodable, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone)]
pub struct Market {
    pub id: MarketId,
    pub name: String,
    pub price_precision: u32,
    pub size_precision: u32,
    pub base_asset_id: AssetId,
    pub quote_asset_id: AssetId,
}

#[derive(Clone)]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    pub precision: u32,
}

#[derive(Clone)]
pub struct Config {
    pub markets: HashMap<MarketId, Market>,
    pub assets: HashMap<AssetId, Asset>,
}

impl Config {
    pub fn hardcoded() -> Config {
        let mut assets = HashMap::new();
        assets.insert(1, Asset {
            id: 1,
            name: "BTC".to_string(),
            precision: 8,
        });
        assets.insert(1, Asset {
            id: 2,
            name: "USD".to_string(),
            precision: 5,
        });

        let mut markets = HashMap::new();
        markets.insert(1, Market {
            id: 1,
            name: "BTCUSD".to_string(),
            price_precision: 2,
            size_precision: 3,
            base_asset_id: 1,
            quote_asset_id: 2,
        });

        Config {
            assets: assets,
            markets: markets,
        }
    }
}

// An order to buy/sell at the specified or better price for the specified amount
#[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq)]
#[derive(Copy, Clone, Debug)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub market_id: MarketId,
    pub side: OrderSide,
    pub price: OrderPrice,
    pub size: OrderSize,
    pub remaining: OrderSize,
}

impl Order {
    pub fn new(id: OrderId, user_id: UserId, market_id: MarketId, side: OrderSide, price: OrderPrice, size: OrderSize) -> Order {
        Order {
            id: id,
            user_id: user_id,
            market_id: market_id,
            side: side,
            price: price,
            size: size,
            remaining: size,
        }
    }
}

// A trade is a match between a bid and an ask
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Trade {
    pub market_id: MarketId,
    pub price: OrderPrice,
    pub size: OrderSize,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub maker_user_id: UserId,
    pub taker_user_id: UserId,
    // Which side the maker was on
    pub side: OrderSide,
}

impl Trade {
    pub fn new(maker: &Order, taker: &Order) -> Trade {
        Trade {
            price: maker.price,
            size: cmp::min(maker.remaining, taker.remaining),
            maker_order_id: maker.id,
            taker_order_id: taker.id,
            maker_user_id: maker.user_id,
            taker_user_id: taker.user_id,
            side: maker.side,
            market_id: maker.market_id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseDecimalError {
    BadIntegerPart,
    BadDecimalPart,
    ExtraDecimalPoint,
    PrecisionTooHigh,
}

pub fn parse_decimal(input: &str, decimal_places: u32, allow_precision_loss: bool) -> Result<u64, ParseDecimalError> {
    use self::ParseDecimalError::*;

    let mut substrs = input.split('.').fuse();

    let integer_part: u64 = try!(
        str::parse(substrs.next().unwrap()).map_err(|_| BadIntegerPart)
    );

    let (mut decimal_part, decimal_width): (u64, u32) = match substrs.next() {
        None => (0, 1),
        Some("") => (0, 1),
        Some(s) => {
            (try!(str::parse(s).map_err(|_| BadDecimalPart)), s.len() as u32)
        }
    };

    if substrs.next().is_some() {
        return Err(ExtraDecimalPoint);
    }

    let shift_distance = decimal_places as i32 - decimal_width as i32;
    let factor = 10u64.pow(shift_distance.abs() as u32);

    if shift_distance >= 0 {
        decimal_part *= factor;
    } else {
        if !allow_precision_loss {
            return Err(PrecisionTooHigh);
        }
        decimal_part /= factor;
    }

    Ok(10u64.pow(decimal_places) * integer_part + decimal_part)
}
