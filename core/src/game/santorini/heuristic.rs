use game::santorini::*;

pub const BEST : HeuristicValue = 100;

pub const WORST : HeuristicValue = -100;

pub const PLAYER_0_WIN : HeuristicValue = 100;
pub const PLAYER_1_WIN : HeuristicValue = -100;

pub struct SimpleHeightHeuristic {}

impl Heuristic for SimpleHeightHeuristic {
    #[allow(unused_variables)]
    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue {
        let mut n : HeuristicValue = 0;

        for &bl in &state.builder_locations[0] {
            if StandardBoard::valid(bl) {
                let h = state.buildings.get(bl) as HeuristicValue;
                n += h;
            } else {
                return 0;
            }
        }

        for &bl in &state.builder_locations[1] {
            if StandardBoard::valid(bl) {
                let h = state.buildings.get(bl) as HeuristicValue;
                n -= h;
            } else {
                return 0;
            }
        }

        n
    }
}