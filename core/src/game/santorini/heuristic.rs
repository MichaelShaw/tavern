use game::santorini::*;

pub type HeuristicValue = i8;

pub const BEST : HeuristicValue = 100;
pub const WORST : HeuristicValue = -100;

trait Heuristic {
	fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue;
}