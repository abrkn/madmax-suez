use std::fs::{File, OpenOptions };
use std::io::{Write, BufWriter};
use bincode::serde::{serialize_into, deserialize_from, DeserializeError};
use utils::*;
use messages::*;

pub trait JournalWriter {
    fn write(&mut self, message: &super::messages::Message) -> Result<(), String>;
}

pub struct JsonJournalWriter {
    file: File,
}

impl JsonJournalWriter {
    pub fn new(filename: &str) -> Result<JsonJournalWriter, String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(filename);

        match file {
            Ok(file) => Ok(JsonJournalWriter { file: file }),
            Err(err) => Err(err.to_string()),
        }
    }
}

impl JournalWriter for JsonJournalWriter {
    fn write(&mut self, message: &super::messages::Message) -> Result<(), String> {
        let mut serialized = serde_json::to_string(&message).unwrap().into_bytes();
        serialized.push(10);

        match self.file.write_all(&serialized) {
            Ok(()) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
        // TODO: sync_data
    }
}

pub struct BinaryJournalWriter {
    writer: BufWriter<File>,
}

impl BinaryJournalWriter {
    pub fn new(filename: &str) -> Result<BinaryJournalWriter, String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(filename);

        match file {
            Ok(file) => Ok(BinaryJournalWriter {
                writer: BufWriter::new(file),
            }),
            Err(err) => Err(err.to_string()),
        }
    }
}

impl JournalWriter for BinaryJournalWriter {
    fn write(&mut self, message: &super::messages::Message) -> Result<(), String> {
        match serialize_into(&mut self.writer, &message, bincode::SizeLimit::Infinite) {
            Ok(()) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
        // TODO: flush to disk
    }
}

// use std::fs::File;
use std::io::{BufReader, BufRead, Lines};

pub struct JsonJournalReader {
    iter: Lines<BufReader<File>>,
}

impl JsonJournalReader {
    pub fn new(filename: &str) -> JsonJournalReader {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        JsonJournalReader {
            iter: reader.lines(),
        }
    }
}

impl Iterator for JsonJournalReader {
    type Item = Result<Message, String>;

    fn next(&mut self) -> Option<Result<Message, String>> {
        match self.iter.next() {
            None => None,
            Some(line) => {
                let unwrapped_line = line.unwrap();
                let parsed_json: Message = serde_json::from_str(&unwrapped_line).unwrap();
                Some(Ok(parsed_json))
            }
        }
    }
}

pub struct BinaryJournalReader {
    reader: BufReader<File>,
}

impl BinaryJournalReader {
    pub fn new(filename: &str) -> BinaryJournalReader {
        let file = File::open(filename).unwrap();

        BinaryJournalReader {
            reader: BufReader::new(file),
        }
    }
}

impl Iterator for BinaryJournalReader {
    type Item = Result<Message, String>;

    fn next(&mut self) -> Option<Result<Message, String>> {
        match deserialize_from::<_, Message>(&mut self.reader, bincode::SizeLimit::Infinite) {
            Ok(message) => Some(Ok(message)),
            Err(err) => {
                match err {
                    DeserializeError::EndOfStreamError => None,
                    _ => Some(Err(err.to_string())),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;
    use std::fs::{remove_file};
    use std::fs::{File, OpenOptions };
    use std::io::{Read};
    use bincode::rustc_serialize::{encode, decode};
    use utils::*;
    use super::super::messages::{MessagePayload};

    #[test]
    fn it_adds_bids_in_correct_order_from_json() {
        let filename = "journal-1.json";

        if fs::metadata(filename).is_ok() {
            remove_file(filename).unwrap();
        }

        {
            let mut journaler = JsonJournalWriter::new(filename).unwrap();
            journaler.write(&super::super::messages::Message {
                sequence: 1,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 1,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        let reader = JsonJournalReader::new(filename);
        let decoded = reader.map(|x| x.unwrap()).next().unwrap();

        match decoded.payload {
            super::super::messages::MessagePayload::CreateOrder(order) => {
                assert_eq!(order.id, 1);
                assert_eq!(order.user_id, 2);
                assert_eq!(order.market_id, 3);
                assert_eq!(order.side, OrderSide::Buy);
                assert_eq!(order.price, 100);
                assert_eq!(order.size, 50);
                assert_eq!(order.remaining, 50);
            },
            _ => panic!("incorrect type"),
        }
    }

    #[test]
    fn it_can_read_json_journal() {
        let filename = "journal-2.json";

        {

            let mut journaler = JsonJournalWriter::new(filename).unwrap();
            journaler.write(&super::super::messages::Message {
                sequence: 1,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 1,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 2,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 2,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        let mut reader = JsonJournalReader::new(filename);

        let message_1 = reader.next().unwrap().unwrap();
        assert_eq!(message_1.sequence, 1);

        let message_2 = reader.next().unwrap().unwrap();
        assert_eq!(message_2.sequence, 2);
    }

    #[test]
    fn it_appends_json() {
        let filename = "journal-3.json";

        if fs::metadata(filename).is_ok() {
            remove_file(filename).unwrap();
        }

        {
            let mut journaler = JsonJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 1,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 1,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        {
            let mut journaler = JsonJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 2,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 2,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        let mut reader = JsonJournalReader::new(filename);

        let message_1 = reader.next().unwrap().unwrap();
        assert_eq!(message_1.sequence, 1);

        let message_2 = reader.next().unwrap().unwrap();
        assert_eq!(message_2.sequence, 2);
    }

    #[test]
    fn it_appends_binary() {
        let filename = "journal-append.dat";

        if fs::metadata(filename).is_ok() {
            remove_file(filename).unwrap();
        }

        {
            let mut journaler = BinaryJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 1,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 1,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        {
            let mut journaler = BinaryJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 2,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 2,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        let mut reader = BinaryJournalReader::new(filename);

        let message_1 = reader.next().unwrap().unwrap();
        assert_eq!(message_1.sequence, 1);

        let message_2 = reader.next().unwrap().unwrap();
        assert_eq!(message_2.sequence, 2);

        match message_2.payload {
            MessagePayload::CreateOrder(order) => {
                assert_eq!(order.price, 100);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn it_handles_binary_eof() {
        let filename = "journal-binary-eof.dat";

        if fs::metadata(filename).is_ok() {
            remove_file(filename).unwrap();
        }

        {
            let mut journaler = BinaryJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 1,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 1,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        {
            let mut journaler = BinaryJournalWriter::new(filename).unwrap();

            journaler.write(&super::super::messages::Message {
                sequence: 2,
                payload: super::super::messages::MessagePayload::CreateOrder(Order {
                    id: 2,
                    market_id: 3,
                    side: OrderSide::Buy,
                    price: 100,
                    size: 50,
                    remaining: 50,
                    user_id: 2,
                }),
            }).unwrap();
        }

        let mut reader = BinaryJournalReader::new(filename);

        reader.next().unwrap().unwrap();
        reader.next().unwrap().unwrap();

        let n = reader.next();

        println!("{:?}", n);

        assert!(n.is_none());
    }
}
