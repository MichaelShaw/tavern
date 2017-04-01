use game::santorini::*;
// use game::*;
use game::util::Packed;
use game::packed::*;
use std;
use std::cmp::{min};

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

        for bl in state.builders[0].iter() {
            if StandardBoard::valid(bl) {
                let h = state.get_building_height(bl) as HeuristicValue;
                n += h;
            } else {
                return 0;
            }
        }

        for bl in state.builders[1].iter() {
            if StandardBoard::valid(bl) {
                let h = state.get_building_height(bl) as HeuristicValue;
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

        let collision = state.collision();

        // we can do this better with height maps and counting zeroes

        for bl in state.builders[player.0 as usize].iter() {
            if StandardBoard::valid(bl) {
                let current_height = state.get_building_height(bl);
                n += (current_height + 1) as HeuristicValue;
                for &move_to in board.adjacencies[bl.0 as usize].iter() {
                    if move_to == NONE { // we've reached end of adjacencies
                        break;
                    }
                    let target_height = state.get_building_height(move_to);

                    if collision.get(move_to) == 0 && target_height <= current_height + 1 {
                        n += (target_height + 1) as HeuristicValue;
                    }
                }
            }
        }

        n
    }
}

pub struct AdjustedNeighbourHeuristic { }

impl Heuristic for AdjustedNeighbourHeuristic {
    fn name() -> String {
        "AdjustedNeighbourHeuristic".into()
    }

    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue {
        let heights = state.building_map();

        let available = !state.collision();

        Self::freedom_for(board, state, Player(0), &heights, available) - 
          Self::freedom_for(board, state, Player(1), &heights, available)
    }
}


impl AdjustedNeighbourHeuristic {
    fn freedom_for(board: &StandardBoard, state: &State, player:Player, heights: &[Packed1; 4], available: Packed1) -> HeuristicValue {
        let mut n : HeuristicValue = 0;

        let builders = state.builders[player.0 as usize];

        

        let mut builder_adjacencies = [
            PACKED1_EMPTY, // anyone can move to height 0
            PACKED1_EMPTY, // anyone can move to height 1
            PACKED1_EMPTY,
            PACKED1_EMPTY,
        ];

        let mut shared_adjacencies = PACKED1_EMPTY;

        // println!("AH:: player {:?}", player);

        // 1. exact heights
        for bl in builders.iter() {
            let h = state.get_building_height(bl);

            // add current height o value
            let hv = STANDING_SCORE[h as usize];

            // println!("AH :: builder {:?} gets {} for height", bl, hv);
            n += hv;



            let adjacencies = board.packed_adjacencies[bl.0 as usize];

            shared_adjacencies |= adjacencies;

            // imprint on accessability
            let max_height = min(h + 1, 3);
            for h in 2..(max_height+1) {
                builder_adjacencies[h as usize].0 |= adjacencies.0;
            }
        }

        // anyone can move to height 0 or 1
        builder_adjacencies[0] = shared_adjacencies;
        builder_adjacencies[1] = shared_adjacencies;
        
        for h in 0..4 {
            let neighbour_count = (available & builder_adjacencies[h] & heights[h]).count() as HeuristicValue;
            let hv = neighbour_count * NEIGHBOUR_SCORE[h as usize];

            // println!("AH :: height {} with square count {} gets {}", h, neighbour_count, hv);

            n += hv;
        }


        n
    }
}

pub const STANDING_SCORE : [HeuristicValue; 4] = [0, 2, 8, 2]; // the 2 for height 3 is because ... height 3 is worthless outside of movement freedom
pub const NEIGHBOUR_SCORE : [HeuristicValue; 4] = [1, 2, 4, 8];


#[cfg(test)]
mod tests {
    // use game::santorini::*;
    use super::*;
    use super::super::tests::*;

    #[test]
    fn adj_neighbour_heuristic() {
        let board = StandardBoard::new(ZobristHash::new_unseeded());

        println!("INITIAL -> {}", AdjustedNeighbourHeuristic::evaluate(&board, &INITIAL_STATE));

        let mut state = INITIAL_STATE;
        for &mve in &vec![Move::PlaceBuilders { a: Slot(3), b: Slot(1) }, 
                          Move::PlaceBuilders { a: Slot(23), b: Slot(24) }] {
            state = board.apply(mve, &state);
        }
        state.set_building_height(Slot(1), 2);
        state.set_building_height(Slot(2), 3);
        state.to_move = Player(0);

        println!("state -> {}", board.print(&state));
        println!("a in 1 -> {}", AdjustedNeighbourHeuristic::evaluate(&board, &state));
    }
}