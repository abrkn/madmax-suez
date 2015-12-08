#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![feature(plugin)]
#[macro_use]
extern crate bincode;
// extern crate rustc_serialize;
extern crate websocket;

extern crate serde;
extern crate serde_json;


// TODO: bid->buy

extern crate lazy_static;
extern crate zmq;

// pub mod utils;

// mod matcher;
// mod balances;
// mod journaler;

extern crate rand;
extern crate rustc_serialize;

// use rustc_serialize::json::{self, Json};
// use rand::Rng;
// use rand::distributions::{IndependentSample, Range};
// use rand::distributions::exponential::{Exp};
// use rand::distributions::normal::{Normal};
// use matcher::Matcher;
// use journaler::Journaler;
// use balances::Balances;
// use utils::*;
// use std::thread;
// use std::str::from_utf8;
// use websocket::{Server, Message, Sender, Receiver};
// use websocket::server::Connection;
// use websocket::stream::WebSocketStream;
// use websocket::message::Type;
// use std::borrow::Cow;
// use std::collections::BTreeMap;
// use std::collections::HashMap;
// use rustc_serialize::json::ToJson;
// use std::collections::Vec;

mod utils;
mod messages;
mod sequencer;
mod matcher;
mod balances;

use utils::{Config, Order, ORDER_SIDE_BID, ORDER_SIDE_ASK};
use messages::{Message, MessagePayload, OrderSide};
use matcher::{Book};
use balances::{Balances};
use uuid::Uuid;

struct Machine {
  config: Config,
  book: Book,
  balances: Balances,
  sequencer: Sequencer,
}

pub enum MachineApplyError {
  InsufficientFunds,
}

impl Machine {
  fn new() -> Machine {
    Machine {
      config: Config::hardcoded(),
      book: Book::new(),
      balances: Balances::new(Config::hardcoded()),
      order_id_counter: 0,
    }
  }

  fn apply(&mut self, message: &Message) -> Result<String, String> {
    match &message.payload {
      &MessagePayload::CreateOrder { ref side, ref price, ref size } => {
        let mut order = Order {
          id: Uuid::new_v4().to_hyphenated_string(), // TODO: Elsewhere
          side: match *side { OrderSide::Buy => ORDER_SIDE_BID, _ => ORDER_SIDE_ASK },
          size: *size,
          price: *price,
          remaining: *size,
          market_id: 1,
          user_id: 1,
        };









        println!("{:?}", order);

        if Err(x) let self.balances.debit_for_order(&order) {
          return x;
        }


        // TODO:

        self.book.execute_order(order);

        unimplemented!();
      },

      _ => unimplemented!(),
    }

    unimplemented!();
  }
}

fn main() {
  let mut machine = Machine::new();





  machine.apply(&Message {
    sequence: 1,
    created_at: "temp".to_string(),
    payload: MessagePayload::CreateOrder {
      side: OrderSide::Buy,
      price: 1,
      size: 2,
    },
  });
}


// use num::traits::{PrimInt};



// fn execute_random_orders() {
//     let config = Config::hardcoded();
//     let mut market = Book::new();
//     let mut rng = rand::thread_rng();
//     let price_exp = Normal::new(1000.0, 20.0);
//     let size_exp = Normal::new(10.0, 2.5);
//     // let mut combined_trades = Vec::<Trade>::new();
//     let mut journaler = Journaler::new().unwrap();
//     let mut balances = Balances::new(config);
//     let mut trade_count = 0;

//     balances.balances.insert((1, 1), 10000000000);
//     balances.balances.insert((1, 2), 10000000000);

//     for order_id in 1..1000 * 100 {
//         let side = if rng.gen() { ORDER_SIDE_BID } else { ORDER_SIDE_ASK };
//         let mut price = (price_exp.ind_sample(&mut rng)).abs() as u64;
//         let size = (size_exp.ind_sample(&mut rng)).abs() as u64;

//         if side == ORDER_SIDE_BID {
//             price = price - 20;
//         } else {
//             price = price + 20;
//         }

//         let order = Order::new(order_id, 1, 1, side, price, size);
//         balances.debit_for_order(&order).unwrap();

//         let action = Action::CreateOrder(order);
//         journaler.write_action(&action).unwrap();

//         // println!("{} {} @ {}", if side == ORDER_SIDE_BID { "bid"} else { "ask" }, size, price);

//         let mut trades = market.execute_order(order);

//         trade_count += trades.len();

//         if order_id % 1000 == 0 {
//             println!("placed {}", order_id);
//         }

//         // while trades.len() > 0 {
//         //     combined_trades.push(trades.remove(0));
//         // }
//     }

//     // println!("total matches: {}", combined_trades.len());
//     println!("total matches: {}", trade_count);
// }

// fn parse_json_string_number(value: &str, scale: u32) -> u64 {
//     let scaled = value.parse::<f64>().unwrap() * (10u32).pow(scale) as f64;
//     // if (scaled % 1 != 0) {
//     //     panic!("precision too high");
//     // }
//     scaled as u64
// }

// fn replay(filename: &str) {
//     use std::fs::File;
//     use std::io::{BufReader, BufRead};

//     let mut book = Book::new();

//     let mut id = 0;
//     let f = File::open(filename).unwrap();
//     let mut reader = BufReader::new(&f);

//     let mut journaler = Journaler::new().unwrap();
//     let mut balances = Balances::new(Config::hardcoded());
//     let mut trade_count = 0;

//     balances.balances.insert((1, 1), 10000000000000);
//     balances.balances.insert((1, 2), 10000000000000);

//     for line in reader.lines().map(|x| x.unwrap()) {
//         println!("{}", line);
//         let json = Json::from_str(&line).unwrap();
//         let obj = json.as_object().unwrap();
//         let message_type = obj.get("type").unwrap().as_string().unwrap();
//         if (message_type != "received") { continue; }
//         if !obj.get("funds").unwrap().is_null() { continue; }
//         if obj.get("price").unwrap().is_null() { continue; }
//         let size = parse_decimal(obj.get("size").unwrap().as_string().unwrap(), 3, true).unwrap();
//         let price = parse_decimal(obj.get("price").unwrap().as_string().unwrap(), 2, true).unwrap();
//         let side = if obj.get("side").unwrap().as_string().unwrap() == "buy" {ORDER_SIDE_BID } else { ORDER_SIDE_ASK };
//         id = id + 1;

//         let order = Order {
//             id: id,
//             market_id: 1,
//             user_id: 1,
//             side: side,
//             price: price,
//             size: size,
//             remaining: size,
//         };

//         let order_side_name = obj.get("side").unwrap().to_string();

//         println!("{} {} BTC @ {} USD", order_side_name, order.size as f64 * 0.001, order.price as f64 * 0.01);

//         balances.debit_for_order(&order).unwrap();

//         let action = Action::CreateOrder(order);
//         journaler.write_action(&action).unwrap();

//         let mut trades = book.execute_order(order);
//         trade_count += trades.len();

//         // if trades.len() > 0 {
//         //     println!("{} trades", trades.len());
//         // }

//         // if (id > 1000) { break; }
//     }
// }

// fn main() {
//     let filename = (std::env::var("PWD").unwrap() + "/../coinbase-recorder/coinbase-2015-11-23-16-18.log");
//     println!("filename {}", filename);
//     replay(&filename);
// }

fn run_server() {
    let config = Config::hardcoded();

    let mut journaler = Journaler::new().unwrap();
    let mut book = Book::new();
    let mut balances = Balances::new(Config::hardcoded());
    let mut sequencer = Sequencer::new();
    let mut machine = Machine::new();

    let addr = "127.0.0.1:9001".to_string();
    let server = Server::bind(&addr[..]).unwrap();

    for connection in server {
        let request = connection.unwrap().read_request().unwrap();
        request.validate().unwrap();
        let response = request.accept();
        let (mut sender, mut receiver) = response.send().unwrap().split();

        for message in receiver.incoming_messages() {
            let message: Message = match message {
                Ok(message) => message,
                Err(e) => {
                    println!("{:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            };

            let json;
            let message = match message.opcode {
                Type::Text => {
                    let raw = from_utf8(&*message.payload).unwrap();
                    json = Json::from_str(raw).unwrap();
                    json.as_object().unwrap()
                },
                _ => unimplemented!(),
            };

            let method = message.get("method").unwrap().as_string().unwrap();
            let params = message.get("params").unwrap().as_array().unwrap();
            let id = message.get("id").unwrap().as_u64().unwrap();

            match method {
                "createOrder" => {
                    let desc = params.get(0).unwrap().as_object().unwrap();
                    let size = parse_decimal(desc.get("size").unwrap().as_string().unwrap(), 3);
                    let price = parse_decimal(desc.get("price").unwrap().as_string().unwrap(), 2);
                    let side = if match desc.get("side").unwrap().as_string().unwrap() = "buy" { OrderSide::Buy } else { OrderSide::Sell };

                    let mut message = {
                      sequence: 0,
                      created_at: "temp".to_string(),
                      payload: MessagePayload::CreateOrder(Order {

                      })
                    };

                    if Err(x) let self.balances.debit_for_order(&order) {
                      return x;
                    }








                    let order = Order {
                        price: price,
                        size: size,
                        side: side,
                        user_id: 1,
                        market_id: 1,
                        id: order_id,
                        remaining: size,
                    };

                    let order_side_name = match order.side {
                        ORDER_SIDE_BID => "buy".to_string(),
                        _ => "sell".to_string(),
                    };

                    println!("{} {} BTC @ {} USD", order_side_name, order.size as f64 * 0.001, order.price as f64 * 0.01);

                    let mut trades = book.execute_order(order);

                    if trades.len() > 0 {
                        println!("{} trades", trades.len());
                    }

                    let mut response = BTreeMap::new();
                    response.insert("id".to_string(), id.to_json());

                    let mut response_result = BTreeMap::new();
                    response_result.insert("id".to_string(), order_id.to_json());

                    response.insert("result".to_string(), response_result.to_json());

                    let response = json::encode(&response).unwrap();
                    let response = Message::text(response);
                    sender.send_message(&response).unwrap();
                },
                "cancelOrder" => {
                    let order_id = params.get(0).unwrap().as_u64().unwrap();
                    let canceled_size = book.cancel_order(order_id);

                    println!("canceled {} BTC", canceled_size);

                    let mut response = BTreeMap::new();
                    response.insert("id".to_string(), id.to_json());

                    let mut response_result = BTreeMap::new();
                    response_result.insert("canceled_size".to_string(), canceled_size.to_json());

                    response.insert("result".to_string(), response_result.to_json());

                    let response = json::encode(&response).unwrap();
                    let response = Message::text(response);
                    sender.send_message(&response).unwrap();
                },
                _ => unimplemented!(),
            }
        }
    }
}
