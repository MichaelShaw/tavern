
use game::santorini::*;

use std::cmp::max;

pub struct NegaMax {

}

impl NegaMax {
    // THIS IS 100% FUCKED
    pub fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> Vec<(Move, HeuristicValue)> where H: Heuristic {
        let color : i8 = match state.to_move {
            Player(0) => 1,
            Player(1) => -1,
            _ => -98,
        };

        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);
        let unsorted_moves : Vec<_> = moves.iter().map(|&mve| {
            if board.ascension_winning_move(state, mve) {
                (mve, BEST)
            } else {
                let new_state = board.apply(mve, state);
                (mve, NegaMax::eval::<H>(board, &new_state, depth - 1, color) * color) // 
            }
        }).collect();
        unsorted_moves
    }

    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, color: i8) -> HeuristicValue where H: Heuristic {
        if depth == 0 {
            return H::evaluate(board, state) * color;
        }

        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        if moves.is_empty() {
           return WORST;
        }

        let mut best_observed = WORST;

        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                return BEST; // unsure of this one
            } else {
                let new_state = board.apply(mve, state);
                let v = -NegaMax::eval::<H>(board, &new_state, depth - 1, -color);
                best_observed = max(v, best_observed);
            }
        }

        best_observed
    }
}