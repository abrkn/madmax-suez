use std::str;
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;

use rustc_serialize::json::{self, Json, ToJson};
use websocket;
use websocket::stream::{WebSocketStream};
use websocket::ws::sender::Sender;
use websocket::ws::receiver::Receiver;

use utils::*;
use balances::*;
use engine::*;
use messages::*;
use journal::*;

pub struct SuezServerReceiver {
    send_tx: mpsc::Sender<String>,
    engine_tx: mpsc::Sender<Message>,
}

impl SuezServerReceiver {
    fn handle_create_order(&mut self, params: &Vec<Json>) -> Result<Json, String> {
        let desc = params.get(0).unwrap().as_object().unwrap();
        let size = parse_decimal(desc.get("size").unwrap().as_string().unwrap(), 3, true);
        let price = parse_decimal(desc.get("price").unwrap().as_string().unwrap(), 2, true);
        let side = if desc.get("side").unwrap().as_string().unwrap() == "buy" { OrderSide::Buy } else { OrderSide::Sell };
        let order_id = time::precise_time_ns();

        let payload = Order {
            id: order_id,
            market_id: 1,
            user_id: 1,
            side: side,
            price: price.unwrap(),
            size: size.unwrap(),
            remaining: size.unwrap(),
        };

        let message = Message {
            sequence: 0,
            payload: MessagePayload::CreateOrder(payload),
        };

        self.engine_tx.send(message).unwrap();

        let mut response = BTreeMap::new();
        response.insert("order_id".to_string(), payload.id.to_json());

        Ok(Json::Object(response))
    }

    fn handle_cancel_order(&mut self, params: &Vec<Json>) -> Result<Json, String> {
        let desc = params.get(0).unwrap().as_object().unwrap();
        let order_id = desc.get("size").unwrap().as_u64().unwrap();
        // TODO: user_id

        let message = Message {
            sequence: 0,
            payload: MessagePayload::CancelOrder {
                order_id: order_id,
            },
        };

        self.engine_tx.send(message).unwrap();

        let response = BTreeMap::new();
        Ok(Json::Object(response))
    }

    fn handle_adjust_balance(&mut self, params: &Vec<Json>) -> Result<Json, String> {
        let desc = params.get(0).unwrap().as_object().unwrap();
        let amount = parse_decimal(desc.get("amount").unwrap().as_string().unwrap(), 10, true).unwrap();
        let user_id = 1; // TODO
        let asset_id = if desc.get("asset").unwrap().as_string().unwrap() == "BTC" { 1 } else { 2 };

        let message = Message {
            sequence: 0,
            payload: MessagePayload::AdjustBalance {
                user_id: user_id,
                asset_id: asset_id,
                change: amount as i64,
            },
        };

        self.engine_tx.send(message).unwrap();

        let response = BTreeMap::new();

        Ok(Json::Object(response))
    }

    fn handle_json_message(&mut self, message: BTreeMap<String, Json>) {
        let method = message.get("method").unwrap().as_string().unwrap();
        let params = message.get("params").unwrap().as_array().unwrap();
        let id = message.get("id").unwrap().as_u64().unwrap();

        let response = match method {
            "createOrder" => self.handle_create_order(params),
            "adjustBalance" => self.handle_adjust_balance(params),
            "cancelOrder" => self.handle_cancel_order(params),
            _ => unimplemented!(),
        };

        let response = match response {
            Ok(json) => {
                let mut response = BTreeMap::new();
                response.insert("id".to_string(), id.to_json());
                response.insert("result".to_string(), json);
                response
            },
            Err(err) => {
                let mut response = BTreeMap::new();
                response.insert("id".to_string(), id.to_json());
                // response.insert("error".to_string(), err.to_json());
                response
            },
        };

        let response = json::encode(&response).unwrap();
        self.send_tx.send(response).unwrap();


        // let response = websocket::message::Message::text(response);
        // sender.send_message(&response).unwrap();
    }

    fn handle_websocket_message(&mut self, message: websocket::message::Message) {

        match message.opcode {
            websocket::message::Type::Text => {
                // println!("incoming: {:?}", str::from_utf8(&*message.payload).unwrap());
                // self.send_tx.send(str::from_utf8(&*message.payload).unwrap().to_string()).unwrap();
                self.handle_json_message(Json::from_str(str::from_utf8(&*message.payload).unwrap()).unwrap().as_object().unwrap().to_owned());
            },
            _ => unimplemented!(),
        };
    }

    fn run(mut self, mut receiver: websocket::server::Receiver<WebSocketStream>) {
        for message in receiver.incoming_messages() {
            self.handle_websocket_message(message.unwrap());
        }
    }
}

pub struct SuezServer {
    balances: Balances,
    engine_channel: mpsc::Sender<Message>,
    senders: Vec<mpsc::Sender<String>>,
}

impl SuezServer {
    pub fn new() -> SuezServer {
        let mut balances = Balances::new(Config::hardcoded());
        balances.adjust_balance(1, 1, 10000000000000000);
        balances.adjust_balance(1, 2, 10000000000000000);

        let engine_channel = SuezEngine::<JsonJournalWriter>::start(balances.clone());
        println!("engine created");

        SuezServer {
            balances: balances,
            engine_channel: engine_channel,
            senders: vec![],
        }
    }

    fn handle_connection(&mut self, connection: websocket::server::Connection<WebSocketStream, WebSocketStream>) {
        let request = connection.read_request().unwrap();
        request.validate().unwrap();

        let response = request.accept();
        let client = response.send().unwrap();

        let (mut ws_sender, ws_receiver) = client.split();

        let (send_tx, send_rx) = mpsc::channel::<String>();

        let balances = self.balances.clone();
        let engine_tx = self.engine_channel.clone();

        self.senders.push(send_tx.clone());

        let receive_thread = thread::spawn(move || {
            let receiver = SuezServerReceiver {
                // balances: balances,
                engine_tx: engine_tx,
                send_tx: send_tx,
            };

            receiver.run(ws_receiver);
        });

        let send_thread = thread::spawn(move || {
            loop {
                let message_text = send_rx.recv().unwrap();
                println!("im supposed to send something! {}", message_text);
                let message = websocket::message::Message::text(message_text);
                ws_sender.send_message(&message).unwrap();
            }
        });

        receive_thread.join().unwrap();
        send_thread.join().unwrap();
    }

    // TODO: Multiple connections
    pub fn listen(mut self, addr: &str) {
        let server = websocket::Server::bind(&addr[..]).unwrap();

        for connection in server {
            self.handle_connection(connection.unwrap());
        }
    }
}
