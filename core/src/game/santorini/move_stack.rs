

use game::santorini::*;

pub const MAX_DEPTH: usize = 15;
pub const MAX_MOVES: usize = 256;
pub const MAX_MOVE_STACK : usize = MAX_DEPTH * MAX_MOVES;

pub struct MoveStack {
    pub moves: [Move; MAX_MOVE_STACK],
    pub next: usize,
}

impl MoveStack {
    pub fn new() -> MoveStack {
        MoveStack {
            moves: [Move::PlaceBuilders { a: Slot(0), b: Slot(0) } ; MAX_MOVE_STACK],
            next: 0,
        }
    }

    pub fn push(&mut self, mve:Move) {
       self.moves[self.next] = mve;
       self.next += 1;
    }
}

impl MoveSink for MoveStack {
    fn sink(&mut self, mve:Move) {
        self.push(mve);
    }
}