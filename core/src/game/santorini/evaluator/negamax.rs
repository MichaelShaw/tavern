
use game::santorini::*;
use std::cmp::max;


pub struct NegaMax { }

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

impl Evaluator for NegaMax {
    type EvaluatorState = ();

    fn name() -> String {
        "NegaMax".into()
    }
    
    #[allow(unused_variables)]
    fn new_state(board:&StandardBoard) -> () {
        ()
    }

    #[allow(unused_variables)]
    fn evaluate_moves_impl<H>(evaluator_state:  &mut (), board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, EvaluatorInfo) where H: Heuristic {
        let color = color(state.to_move);
        let mut moves = Vec::with_capacity(200);

        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                unsorted_moves.push((mve, BEST * color));
                total_moves += 1;
            } else {
                let new_state = board.apply(mve, state);

                let (v, move_count) = NegaMax::eval::<H>(board, &new_state, depth - 1, -color);
                
                unsorted_moves.push((mve, v * -color));
                total_moves += move_count;
            }
        }
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);
        (unsorted_moves, EvaluatorInfo::from_moves_depth(total_moves, depth))
    }
}

impl NegaMax {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, color: HeuristicValue) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut moves = Vec::with_capacity(200); // enough to prevent resizing
        board.next_moves(state, &mut moves);

        if depth == 0 {
            let v = if moves.is_empty() {
                WORST// * color
            } else {
                H::evaluate(board, state) * color
            };
            return (v, 1);
        }

        let mut total_moves = 0;
        let mut best_observed = WORST;
        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                return (BEST, total_moves + 1);
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = NegaMax::eval::<H>(board, &new_state, depth - 1, -color);
                total_moves += move_count;
                best_observed = max(-v, best_observed);
            }
        }

        (best_observed, total_moves)
    }
}