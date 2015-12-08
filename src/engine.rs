use std::fs;
use std::thread;
use std::sync::mpsc;

use journal::*;
use sequencer::*;
use balances::*;
use book::*;
use messages::*;

pub struct SuezEngine<W: JournalWriter> {
    pub sequencer: Sequencer,
    pub journaler: W,
    pub book: Book,
    pub balances: Balances,
}

impl<W: JournalWriter> SuezEngine<W> {
    fn replay(&mut self) {
        if fs::metadata("journal.json").is_err() {
            println!("nothing to replay");
            return;
        }

        println!("replaying");

        // let reader = JsonJournalReader::new("journal.json");
        let reader = BinaryJournalReader::new("journal.binary");

        for message in reader.map(|x| x.unwrap()) {
            assert_eq!(message.sequence, self.sequencer.sequence + 1);
            self.sequencer.sequence += 1;
            self.apply_message(&message);
        }

        println!("replayed to seq {}", self.sequencer.sequence);
    }

    pub fn apply_message(&mut self, message: &Message) {
        match message.payload {
            MessagePayload::CreateOrder(payload) => {
                self.balances.debit_for_order(&payload);

                for trade in self.book.execute_order(payload).iter() {
                    self.balances.settle(&trade);
                }
            },
            MessagePayload::AdjustBalance {
                user_id,
                asset_id,
                change,
            } => {
                self.balances.adjust_balance(user_id, asset_id, change);
            },
            MessagePayload::CancelOrder {
                order_id,
            } => {
                let order = self.book.cancel_order(order_id).unwrap();
                self.balances.credit_for_canceled_order(&order);
            },
            // _ => unimplemented!(),
        }
    }

    pub fn validate(&self, message: &Message) -> Result<(), String> {
        // Validate
        match message.payload {
            MessagePayload::CreateOrder(payload) => {
                if !self.balances.user_can_afford_order(&payload) {
                    // TODO: Real errors
                    return Err("user cannot afford order".to_string());
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub fn process_message(&mut self, mut message: Message) {
        self.sequencer.apply(&mut message);
        self.journaler.write(&message).unwrap();
        self.apply_message(&message);
    }

    pub fn start(balances: Balances) -> mpsc::Sender<Message> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut engine = SuezEngine {
                book: Book::new(),
                sequencer: Sequencer { sequence: 0 },
                // journaler: JsonJournalWriter::new("journal.json").unwrap(),
                journaler: BinaryJournalWriter::new("journal.binary").unwrap(),
                balances: balances,
            };

            engine.replay();

            loop {
                let message: Message = rx.recv().unwrap();

                match engine.validate(&message) {
                    Ok(()) => {
                        let message: Message = rx.recv().unwrap();
                        engine.process_message(message);
                    }
                    Err(err) => {
                        println!("derp err derp: {}", err);
                    }
                }
            }
        });

        tx
    }
}
