
use game::santorini::*;
// use time;
use game::*;

impl StandardBoard {
    pub fn perft(&self, state: &State, depth: usize, move_stack : &mut MoveStack) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut n = 0;

        let stack_begin = move_stack.next;
        self.next_moves(state, move_stack);
        let stack_end = move_stack.next;

        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];

            if self.ascension_winning_move(state, mve) {
                n += 1;
            } else {
                let new_state = self.apply(mve, state);
                n += self.perft(&new_state, depth - 1, move_stack);
            }
        }
        
        move_stack.next = stack_begin;

        n
    }

    pub fn transform_packed2(transform: &SlotTransform, packed:[Packed1; 2]) -> [Packed1; 2] {
        let mut out = [PACKED1_EMPTY; 2];

        for i in 0..2 {
            for slot in packed[i as usize].iter() {
                // println!("slot -> {:?}", slot);
                out[i as usize].0 |= 1 << transform.slots[slot.0 as usize].0;
            }    
        }

        out
    }
}

use game::santorini::tests::test_cases;

pub const PERFT_DEPTH : usize = 4;

#[cfg(test)]
mod tests {
    // use game::santorini::*;
    use super::*;

    #[test]
    fn test_perft() {
        let mut move_stack = MoveStack::new();
        let board = StandardBoard::new(ZobristHash::new_unseeded());

        let mut moves = 0;

        let start = time::precise_time_ns();
        for test_case in test_cases(&board) {
            moves += board.perft(&test_case.state, PERFT_DEPTH, &mut move_stack);    
        }
        let duration = time::precise_time_ns() - start;
        let as_seconds = (duration as f64) / 1_000_000_000f64;
        
        let million_moves_per_second = moves as f64 / 1_000_000f64 / as_seconds;
        println!("=== OLD === perft {} moves ({:.2}M/second) in {:.2} seconds", moves, million_moves_per_second, as_seconds);
    }
}