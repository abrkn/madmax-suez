#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate suez;

use suez::server::{SuezServer};

fn main() {
    let suez_server = SuezServer::new();
    suez_server.listen("127.0.0.1:9001");
}
