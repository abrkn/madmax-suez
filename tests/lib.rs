extern crate suez;

use suez::utils::*;
use suez::engine::*;
use suez::book::*;
use suez::sequencer::*;
use suez::journal::*;
use suez::balances::*;
use suez::messages::*;

#[test]
fn it_passes_scenario_1() {
    let balances = Balances::new(Config::hardcoded());
    let book = Book::new();

    // TODO: Self match protection
    // TODO: Tests for engine validation
    // TODO: Should probably do messages with remaining == 0 and have it set after
    // TODO: Remove precision everywhere. 5 for price, 5 for size. 5 + 5 = 10 for totals
    const ALICE_USER_ID: UserId = 1;
    const BOB_USER_ID: UserId = 2;
    const CAROL_USER_ID: UserId = 3;
    const BASE_ASSET_ID: AssetId = 1;
    const QUOTE_ASSET_ID: AssetId = 2;
    const MARKET_ID: MarketId = 1;

    let mut engine = SuezEngine::<JsonJournalWriter> {
        book: book,
        sequencer: Sequencer { sequence: 0 },
        journaler: JsonJournalWriter::new("journal.json").unwrap(),
        balances: balances,
    };

    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::AdjustBalance {
            user_id: ALICE_USER_ID,
            asset_id: BASE_ASSET_ID,
            change: 1000,
        },
    });

    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::AdjustBalance {
            user_id: BOB_USER_ID,
            asset_id: BASE_ASSET_ID,
            change: 1000,
        },
    });

    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::AdjustBalance {
            user_id: BOB_USER_ID,
            asset_id: QUOTE_ASSET_ID,
            change: 1,
        },
    });

    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::AdjustBalance {
            user_id: CAROL_USER_ID,
            asset_id: QUOTE_ASSET_ID,
            change: 35000,
        },
    });

    // Alice: Sell 200 @ 100 (20 000)
    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::CreateOrder(Order {
            id: 1,
            market_id: MARKET_ID,
            user_id: ALICE_USER_ID,
            price: 100,
            size: 200,
            side: OrderSide::Sell,
            remaining: 200,
        }),
    });

    // Bob: Sell 150 @ 100 (15 000)
    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::CreateOrder(Order {
            id: 2,
            market_id: MARKET_ID,
            user_id: BOB_USER_ID,
            price: 100,
            size: 150,
            side: OrderSide::Sell,
            remaining: 150,
        }),
    });

    // Carol: Buy 300 @ 110 (33 000)
    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::CreateOrder(Order {
            id: 3,
            market_id: MARKET_ID,
            user_id: CAROL_USER_ID,
            price: 110,
            size: 300,
            side: OrderSide::Buy,
            remaining: 300,
        }),
    });

    // Confirm book
    assert_eq!(engine.book.bids.len(), 0);
    assert_eq!(engine.book.asks.len(), 1);
    assert_eq!(engine.book.asks[0].price, 100);
    assert_eq!(engine.book.asks[0].size, 150);
    assert_eq!(engine.book.asks[0].remaining, 50);

    // Confirm balances
    // let balances = engine.balances.balances.read().unwrap();
    assert_eq!(engine.balances.get_balance(ALICE_USER_ID, BASE_ASSET_ID), 1000 - 200);
    assert_eq!(engine.balances.get_balance(ALICE_USER_ID, QUOTE_ASSET_ID), 20000);
    assert_eq!(engine.balances.get_balance(BOB_USER_ID, BASE_ASSET_ID), 1000 - 150);
    assert_eq!(engine.balances.get_balance(BOB_USER_ID, QUOTE_ASSET_ID), 1 + 100 * 100);
    assert_eq!(engine.balances.get_balance(CAROL_USER_ID, BASE_ASSET_ID), 300);
    assert_eq!(engine.balances.get_balance(CAROL_USER_ID, QUOTE_ASSET_ID), 35000 - 33000);

    // Bob: Cancel remainder of #2
    // TODO: Test to make sure you cant cancel other peoples orders
    // TODO: Make sure orders exist before cancel
    engine.process_message(Message {
        sequence: 0,
        payload: MessagePayload::CancelOrder { order_id: 2 },
    });

    assert_eq!(engine.book.asks.len(), 0);

    assert_eq!(engine.balances.get_balance(BOB_USER_ID, BASE_ASSET_ID), 1000 - 150 + 50);
}
