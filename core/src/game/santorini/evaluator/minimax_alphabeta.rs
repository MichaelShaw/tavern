
use game::santorini::*;
use std::cmp::{max, min};

pub struct MiniMaxAlphaBeta {
    board: StandardBoard,
}

impl Evaluator for MiniMaxAlphaBeta {
    type EvaluatorState = ();
    
    #[allow(unused_variables)]
    fn new_state(board:&StandardBoard) -> () {
        ()
    }

    #[allow(unused_variables)]
    fn evaluate_moves<H>(evaluator_state:  &mut (), board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
        let mut moves = Vec::with_capacity(200);

        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

        let mut alpha = WORST;
        let mut beta = BEST;
        
        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                let v = if state.to_move == Player(0) {
                    PLAYER_0_WIN
                } else { 
                    PLAYER_1_WIN
                };
                if state.to_move == Player(0) {
                    alpha = max(alpha, PLAYER_0_WIN);
                } else {
                    beta = min(beta, PLAYER_1_WIN);
                }

                unsorted_moves.push((mve, v));
                total_moves += 1;
            } else {
                let new_state = board.apply(mve, state);
                let (val, move_count) = Self::eval::<H>(board, &new_state, depth - 1, alpha, beta);

                if state.to_move == Player(0) {
                    // maximizing pass
                    alpha = max(alpha, val);
                } else {
                    // minimizing pass
                    beta = min(beta, val);
                }

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

impl MiniMaxAlphaBeta {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, alpha: HeuristicValue, beta:HeuristicValue) -> (HeuristicValue, MoveCount) where H: Heuristic {
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
            let mut new_alpha = alpha;
            let mut best_observed = PLAYER_1_WIN; // assume worst case

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return (PLAYER_0_WIN, total_moves + 1);
                } else {
                    let new_state = board.apply(mve, state);
                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, new_alpha, beta);
                    new_alpha = max(v, new_alpha);
                    total_moves += move_count;
                    best_observed = max(v, best_observed);
                    if beta <= alpha {
                        break;
                    }
                }
            }
            
            (best_observed, total_moves)
        } else {
            let mut new_beta = beta;
            let mut best_observed = PLAYER_0_WIN; // assume worst cast

            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    return (PLAYER_1_WIN, total_moves + 1);
                } else {
                    let new_state = board.apply(mve, state);
                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, alpha, new_beta);
                    best_observed = min(v, best_observed);    
                    new_beta = min(new_beta, v);
                    total_moves += move_count;
                    if beta <= alpha {
                        break;
                    }
                }
            }

            (best_observed, total_moves)
        }
    }
}

