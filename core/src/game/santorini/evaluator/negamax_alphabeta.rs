
use game::santorini::*;
use std::cmp::{max, min};

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

pub struct NegaMaxAlphaBeta { }

impl Evaluator for NegaMaxAlphaBeta {
    type EvaluatorState = ();

    fn name() -> String {
        "NegaMaxAlphaBeta".into()
    }

    fn new_state() -> () { () }
    #[allow(unused_variables)]
    fn new_search(evaluator_state: &mut ()) { }
    #[allow(unused_variables)]
    fn reset(evaluator_state: &mut ()) { }
     
    #[allow(unused_variables)]
    fn evaluate_moves_impl<H>(evaluator_state: &mut (), board: &StandardBoard, state: &State, depth: Depth) -> (Option<(Move, HeuristicValue)>, EvaluatorInfo) where H: Heuristic {
    	let color = color(state.to_move);

        let mut moves = Vec::with_capacity(200);
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

    	let mut alpha = WORST;

        for &mve in &moves {
            let v = if board.ascension_winning_move(state, mve) {
                let av = BEST * color;
                alpha = max(alpha, av);
                total_moves += 1;
                av
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, -alpha, -color); // 
                let av = v * -color;
                alpha = max(alpha, -v);
                total_moves += move_count;
                av
            };
            unsorted_moves.push((mve, v));
        }
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);
        (unsorted_moves.first().cloned(), EvaluatorInfo::from_moves_depth(total_moves, depth))
    }
}

impl NegaMaxAlphaBeta {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: Depth, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue) -> (HeuristicValue, MoveCount) where H: Heuristic {
        if depth == 0 {
            return (H::evaluate(board, state) * color, 1);
        }

        let mut moves = Vec::with_capacity(200); // enough to prevent resizing
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut best_observed = WORST;
        let mut new_alpha = alpha;
        for &mve in &moves {
            if board.ascension_winning_move(state, mve) {
                return (BEST, total_moves + 1);
            } else {
                let new_state = board.apply(mve, state);
                let (new_v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, -beta, -new_alpha, -color);
                let v = -new_v;
                total_moves += move_count;
                best_observed = max(v, best_observed);
                new_alpha = max(new_alpha, v);
                if beta <= new_alpha {
                	break;
                }
            }
        }

        (best_observed, total_moves)
    }
}
