use game::santorini::*;
use std;

pub const BEST : HeuristicValue = std::i16::MAX;
pub const WORST : HeuristicValue = -std::i16::MAX; // to prevent overflow on negation

pub const PLAYER_0_WIN : HeuristicValue = std::i16::MAX;
pub const PLAYER_1_WIN : HeuristicValue = -std::i16::MAX; // to prevent overflow on negation

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