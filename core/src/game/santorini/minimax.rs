
use game::santorini::*;
use std::cmp::{max, min};

pub struct MiniMax {}

impl Evaluation for MiniMax {
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> Vec<(Move, HeuristicValue)> where H: Heuristic {
        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        let mut unsorted_moves : Vec<_> = moves.iter().map(|&mve| {
            if board.ascension_winning_move(state, mve) {
                (mve, if state.to_move == Player(0) {
                    PLAYER_0_WIN
                } else { 
                    PLAYER_1_WIN
                })
            } else {
                let new_state = board.apply(mve, state);
                (mve, MiniMax::eval::<H>(board, &new_state, depth - 1)) // 
            }
        }).collect();
        if state.to_move == Player(0) {
            unsorted_moves.sort_by_key(|&(_, hv)| -hv); // maximizing player wants biggest first
            unsorted_moves
        } else {
            unsorted_moves.sort_by_key(|&(_, hv)| hv); // minimizing player wants smallest first
            unsorted_moves
        }
    }

}
impl MiniMax {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8) -> HeuristicValue where H: Heuristic {
        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        if depth == 0 {
            return if moves.is_empty() {
                if state.to_move == Player(0) {
                    PLAYER_1_WIN
                } else {
                    PLAYER_0_WIN
                }
            } else {
                H::evaluate(board, state)
            };
        }


        if state.to_move == Player(0) {
            let mut best_observed = PLAYER_1_WIN; // assume worst case

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return PLAYER_0_WIN;
                } else {
                    let new_state = board.apply(mve, state);
                    let v = MiniMax::eval::<H>(board, &new_state, depth - 1);
                    best_observed = max(v, best_observed);
                }
            }
            
            best_observed
        } else {
            let mut best_observed = PLAYER_0_WIN; // assume worst cast

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return PLAYER_1_WIN;
                } else {
                    let new_state = board.apply(mve, state);
                    let v = MiniMax::eval::<H>(board, &new_state, depth - 1);
                    best_observed = min(v, best_observed);    
                }
            }

            best_observed
        }
    }
}

