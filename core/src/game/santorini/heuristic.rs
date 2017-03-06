use game::santorini::*;

pub type HeuristicValue = i8;

trait Heuristic {
	fn evaluate(board: &StandardBoard, state: &State) -> i8;
}