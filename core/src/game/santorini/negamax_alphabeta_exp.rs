
use game::santorini::*;
use std::cmp::{max, min};

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

pub struct NegaMaxAlphaBetaExp {}

impl Evaluation for NegaMaxAlphaBetaExp {
    // THIS IS 100% FUCKED
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
    	let color = color(state.to_move);
        let mut moves = Vec::with_capacity(200);
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);


    	// let mut alpha = WORST;
        if state.to_move == Player(0) {
            let mut alpha = WORST;
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    let av = BEST * color;
                    unsorted_moves.push((mve, av));

                    alpha = max(alpha, av);

                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);

                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, -alpha, -color);
                    
                    let av = v * -color;

                    alpha = max(alpha, av);

                    unsorted_moves.push((mve, av));
                    total_moves += move_count;
                }
            }
        } else {
            let mut beta = BEST;
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    let av = BEST * color;
                    unsorted_moves.push((mve, av));

                    // alpha = max(alpha, av);
                    beta = min(beta, av);

                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);

                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, beta, -color);
                    
                    let av = v * -color;

                    beta = min(beta, av);

                    // alpha = max(alpha, av);
                    unsorted_moves.push((mve, av));
                    total_moves += move_count;
                }
            }
        }
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);
        (unsorted_moves, total_moves)
    }
}

impl NegaMaxAlphaBetaExp {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut moves = Vec::with_capacity(200); // enough to prevent resizing
        board.next_moves(state, &mut moves);

        if depth == 0 {
            let v = if moves.is_empty() {
                WORST
            } else {
               H::evaluate(board, state) * color
            };
            // let v = H::evaluate(board, state) * color;
            return (v, 1);
        }
       
        let mut total_moves = 0;
        let mut best_observed = WORST;
        let mut new_alpha = alpha;
        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                return (BEST, total_moves + 1);
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, -beta, -new_alpha, -color);
                total_moves += move_count;
                best_observed = max(-v, best_observed);
                new_alpha = max(new_alpha, -v);
                if beta <= new_alpha {
                	break;
                }
            }
        }

        (best_observed, total_moves)
    }
}
