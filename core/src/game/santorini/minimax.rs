
use game::santorini::*;
use std::cmp::{max, min};

pub struct MiniMax {}

impl Evaluation for MiniMax {
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
        let mut moves = Vec::with_capacity(200);
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                unsorted_moves.push((mve, if state.to_move == Player(0) {
                    PLAYER_0_WIN
                } else { 
                    PLAYER_1_WIN
                }));
                total_moves += 1;
            } else {
                let new_state = board.apply(mve, state);
                let (val, move_count) = MiniMax::eval::<H>(board, &new_state, depth - 1);
                total_moves += move_count;
                unsorted_moves.push((mve, val));
            }
        }

        if state.to_move == Player(0) {
            unsorted_moves.sort_by_key(|&(_, hv)| -hv); // maximizing player wants biggest first
        } else {
            unsorted_moves.sort_by_key(|&(_, hv)| hv); // minimizing player wants smallest first
        }
        (unsorted_moves, total_moves)
    }
}

impl MiniMax {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut moves = Vec::with_capacity(200);
        board.next_moves(state, &mut moves);

        if depth == 0 {
            let v = if moves.is_empty() {
                if state.to_move == Player(0) {
                    PLAYER_1_WIN
                } else {
                    PLAYER_0_WIN
                }
            } else {
                H::evaluate(board, state)
            };
            return (v, 1);
        }


        let mut total_moves = 0;

        if state.to_move == Player(0) {
            let mut best_observed = PLAYER_1_WIN; // assume worst case

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return (PLAYER_0_WIN, total_moves + 1);
                } else {
                    let new_state = board.apply(mve, state);
                    let (v, move_count) = MiniMax::eval::<H>(board, &new_state, depth - 1);
                    total_moves += move_count;
                    best_observed = max(v, best_observed);
                }
            }
            
            (best_observed, total_moves)
        } else {
            let mut best_observed = PLAYER_0_WIN; // assume worst cast

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return (PLAYER_1_WIN, total_moves + 1);
                } else {
                    let new_state = board.apply(mve, state);
                    let (v, move_count) = MiniMax::eval::<H>(board, &new_state, depth - 1);
                    best_observed = min(v, best_observed);    
                    total_moves += move_count;
                }
            }

            (best_observed, total_moves)
        }
    }
}

