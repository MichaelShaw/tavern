
use game::santorini::*;
use std::cmp::max;


pub struct NegaMax {}

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

impl Evaluation for NegaMax {
    // THIS IS 100% FUCKED
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
        let color = color(state.to_move);
        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::new();

        // println!("NEGAMAX moves -> {:?}", unsorted_moves);
        if state.to_move == Player(0) {
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    unsorted_moves.push((mve, PLAYER_0_WIN));
                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);

                    let (v, move_count) = NegaMax::eval::<H>(board, &new_state, depth - 1, -color);
                    
                    unsorted_moves.push((mve, -v));
                    total_moves += move_count;
                }
            }
            
            unsorted_moves.sort_by_key(|&(_, hv)| -hv); // big first
        } else {
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    unsorted_moves.push((mve, PLAYER_1_WIN));
                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);
                    let (v, move_count) = NegaMax::eval::<H>(board, &new_state, depth - 1, -color);
                    unsorted_moves.push((mve, v));
                    total_moves += move_count;
                }
            }
            unsorted_moves.sort_by_key(|&(_, hv)| hv);
        }
        (unsorted_moves, total_moves)
    }
}

impl NegaMax {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, color: HeuristicValue) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut moves = Vec::new();
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
            return (color * v, 1);
        }

        let mut total_moves = 0;
        let mut best_observed = WORST;
        for &mve in &moves {
            let (v, move_count)  = if board.ascension_winning_move(state, mve) {
                let wv = if state.to_move == Player(0) { PLAYER_0_WIN } else { PLAYER_1_WIN };
                (wv * color, 1)
            } else {
                let new_state = board.apply(mve, state);
                let (v, mves) = NegaMax::eval::<H>(board, &new_state, depth - 1, -color);

                (-v, mves) 
            };
            total_moves += move_count;

            best_observed = max(v, best_observed);
        }

        (best_observed, total_moves)
    }
}