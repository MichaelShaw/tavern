
use game::santorini::*;

use std::cmp::max;

pub struct Negamax {

}

impl Negamax {
    pub fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> Vec<(Move, HeuristicValue)> where H: Heuristic {
        let color : i8 = match state.to_move {
            Player(0) => 1,
            Player(1) => -1,
            _ => -1,
        };

        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);
        moves.iter().map(|&mve| {
            if board.ascension_winning_move(state, mve) {
                (mve, BEST)
            } else {
                let new_state = board.apply(mve, state);
                (mve, Negamax::eval::<H>(board, &new_state, depth - 1, color) * color)
            }
        }).collect()
    }

    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, color: i8) -> HeuristicValue where H: Heuristic {
        if depth == 0 {
            return H::evaluate(board, state);
        }

        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        if moves.is_empty() {
            return WORST;
        }

        let mut best_observed = WORST;

        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                return BEST;
            } else {
                let new_state = board.apply(mve, state);
                let v = -Negamax::eval::<H>(board, &new_state, depth -1, -color);
                best_observed = max(v, best_observed);
            }
        }

        best_observed
    }
}