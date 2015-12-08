#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;
extern crate bincode;
extern crate rustc_serialize;
extern crate websocket;

pub mod utils;
pub mod messages;
pub mod balances;
pub mod sequencer;
pub mod journal;
pub mod book;
pub mod engine;
pub mod server;

pub use server::{SuezServer};
