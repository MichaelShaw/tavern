use game::santorini::*;

pub type HeuristicValue = i8;

pub const BEST : HeuristicValue = 100;
pub const WORST : HeuristicValue = -100;

trait Heuristic {
	fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue;
}

pub struct SimpleHeightHeuristic {}

impl Heuristic for SimpleHeightHeuristic {
	fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue {
		// for builder_location in state.builder_locations {
			
		// }
		0
	}
}