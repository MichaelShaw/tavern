
use game::santorini::*;
use std::cmp::max;


pub struct NegaMax {}

fn color(player:Player) -> i8 {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

impl Evaluation for NegaMax {
    // THIS IS 100% FUCKED
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> Vec<(Move, HeuristicValue)> where H: Heuristic {
        let color = color(state.to_move);
        let mut moves = Vec::new();
        board.next_moves(state, &mut moves);

        // let mut unsorted_moves : Vec<_> = moves.iter().map(|&mve| {
        //     if board.ascension_winning_move(state, mve) {
        //         let wv = if state.to_move == Player(0) { PLAYER_0_WIN } else { PLAYER_1_WIN };
        //         (mve, wv * color)
        //     } else {
        //         let new_state = board.apply(mve, state);
        //         (mve, NegaMax::eval::<H>(board, &new_state, depth - 1, color))
        //     }
        // }).collect();

        // println!("NEGAMAX moves -> {:?}", unsorted_moves);
        if state.to_move == Player(0) {
            let mut unsorted_moves : Vec<_> = moves.iter().map(|&mve| {
                if board.ascension_winning_move(state, mve) {
                    (mve, PLAYER_0_WIN)
                } else {
                    let new_state = board.apply(mve, state);
                    (mve, -NegaMax::eval::<H>(board, &new_state, depth - 1, -color))
                }
            }).collect();
            unsorted_moves.sort_by_key(|&(_, hv)| -hv); // big first
            unsorted_moves
        } else {
            let mut unsorted_moves : Vec<_> = moves.iter().map(|&mve| {
                if board.ascension_winning_move(state, mve) {
                    (mve, PLAYER_1_WIN)
                } else {
                    let new_state = board.apply(mve, state);
                    (mve, NegaMax::eval::<H>(board, &new_state, depth - 1, -color))
                }
            }).collect();
            unsorted_moves.sort_by_key(|&(_, hv)| hv);
            unsorted_moves
        }
    }
}

impl NegaMax {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, color: i8) -> HeuristicValue where H: Heuristic {
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
            return color * v;
        }

        let mut best_observed = WORST;
        for &mve in &moves {
            let v : HeuristicValue = if board.ascension_winning_move(state, mve) {
                let wv = if state.to_move == Player(0) { PLAYER_0_WIN } else { PLAYER_1_WIN };
                wv * color
            } else {
                let new_state = board.apply(mve, state);
                - NegaMax::eval::<H>(board, &new_state, depth - 1, -color)
            };

            best_observed = max(v, best_observed);
        }

        best_observed
    }
}