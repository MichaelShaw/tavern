
use game::santorini::*;
use std::cmp::max;

pub struct NegaMaxAlphaBeta {}

impl Evaluation for NegaMaxAlphaBeta {
    // THIS IS 100% FUCKED
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
    	(Vec::new(), 0)
    }
}
