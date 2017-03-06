use game::santorini::*;

pub type HeuristicValue = i8;

pub const BEST : HeuristicValue = 100;
pub const WORST : HeuristicValue = -100;

pub trait Heuristic {
    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue;
}

pub struct SimpleHeightHeuristic {}

impl Heuristic for SimpleHeightHeuristic {
    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue {

        let mut n : i8 = 0;
        for &bl in &state.builder_locations[0] {
            if StandardBoard::valid(bl) {
                let h = state.buildings.get(bl) as i8;
                n += h;
            } else {
                return 0;
            }
        }

        for &bl in &state.builder_locations[1] {
            if StandardBoard::valid(bl) {
                let h = state.buildings.get(bl) as i8;
                n -= h;
            } else {
                return 0;
            }
        }

        n
    }
}