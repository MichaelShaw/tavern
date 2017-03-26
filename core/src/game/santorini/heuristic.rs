use game::santorini::*;
use game::*;
use game::util::Packed;
use std;

pub const VICTORY : HeuristicValue = 32_000;

pub const BEST : HeuristicValue = std::i16::MAX;
pub const WORST : HeuristicValue = -std::i16::MAX; // to prevent overflow on negation

pub const PLAYER_0_WIN : HeuristicValue = std::i16::MAX;
pub const PLAYER_1_WIN : HeuristicValue = -std::i16::MAX; // to prevent overflow on negation

pub struct SimpleHeightHeuristic {}

impl Heuristic for SimpleHeightHeuristic {
    fn name() -> String {
        "SimpleHeightHeuristic".into()
    }

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

pub struct NeighbourHeuristic {}

impl Heuristic for NeighbourHeuristic {
    fn name() -> String {
        "NeighbourHeuristic".into()
    }

    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue {
        Self::freedom_for(board, state, Player(0)) - Self::freedom_for(board, state, Player(1))
    }
}

impl NeighbourHeuristic {
    fn freedom_for(board: &StandardBoard, state: &State, player:Player) -> HeuristicValue {
        let mut n : HeuristicValue = 0;

        for &bl in &state.builder_locations[player.0 as usize] {
            if StandardBoard::valid(bl) {
                let current_height = state.buildings.get(bl);
                n += (current_height + 1) as HeuristicValue;
                for &move_to in board.adjacencies[bl.0 as usize].iter() {
                    if move_to == NONE { // we've reached end of adjacencies
                        break;
                    }
                    let target_height = state.buildings.get(move_to);
                    if state.collision.get(move_to) == 0 && target_height <= current_height + 1 {
                        n += (target_height + 1) as HeuristicValue;
                    }
                }
            }
        }

        n
    }
}
