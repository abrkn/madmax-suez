extern crate time;
extern crate serde;
extern crate serde_json;

use utils::*;

#[derive(RustcEncodable, RustcDecodable, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub sequence: u64,
    pub payload: MessagePayload,
}

// TODO: Balances should be able to go negative in case of corrections etc.
#[derive(Serialize, Deserialize, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum MessagePayload {
    CreateOrder(Order),
    CancelOrder {
        order_id: u64,
    },
    AdjustBalance {
        user_id: UserId,
        asset_id: AssetId,
        change: i64,
    }
}
