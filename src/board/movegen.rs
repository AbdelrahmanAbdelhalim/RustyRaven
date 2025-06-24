use crate::types::*;

pub enum GenType {
    Captures,
    Quiets,
    QuietChecks,
    Evasions,
    NonEvasions,
    Legal,
}

struct ExtMove {
    base: Move,
    value: i32,
}

impl ExtMove {
    fn set_from_move(&mut self, m: Move) {
        self.set_from_move(m);
    }
}

struct MoveList {

}