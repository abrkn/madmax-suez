use messages::*;

pub struct Sequencer {
    pub sequence: u64,
}

impl Sequencer {
    pub fn apply(&mut self, message: &mut Message) {
        if message.sequence == 0 {
            message.sequence = self.sequence + 1;
        } else {
            assert_eq!(message.sequence, self.sequence + 1);
        }
        self.sequence += 1;
    }
}
