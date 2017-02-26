// use std::fmt;

// extern crate pad;
pub mod move_builder;

pub use self::move_builder::*;

use super::util::*;
use pad::{PadStr, Alignment};

// magics
pub const UNPLACED_BUILDER : Slot = Slot(255);
pub const DEAD_BUILDER : Slot = Slot(254);
pub const NONE : Slot = Slot(253);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct State {
    pub builder_locations: [[Slot; 2]; 2],
    pub buildings : Packed2, 
    pub domes : Packed1,
    pub collision : Packed1,
    pub to_move : Player,
}

impl State {
    pub fn builders_to_place(&self) -> bool {
        let builders_to_move = self.builder_locations[self.to_move.0 as usize];
        builders_to_move.iter().any(|&pl| pl == UNPLACED_BUILDER )
    }

    pub fn initial() -> State {
        State {
            builder_locations: [[UNPLACED_BUILDER, UNPLACED_BUILDER], [UNPLACED_BUILDER, UNPLACED_BUILDER]],
            buildings: Packed2::empty(),
            domes: Packed1::empty(),
            collision: Packed1::empty(),
            to_move: Player(0),
        }
    }

    pub fn player(&self) -> Player {
        self.to_move
    }

    pub fn next_player(&self) -> Player {
        Player((self.to_move.0 + 1) % 2)
    }

    pub fn build_at(&mut self, slot:Slot) {
        let height = self.buildings.get(slot);
        let build_dome = height == 3;
        if build_dome {
            self.domes = self.domes.set(slot, 1);
            self.collision = self.collision.set(slot, 1);
        } else {
            self.buildings = self.buildings.set(slot, self.buildings.get(slot) + 1);
        }
    }
}

const PLAYERS : usize = 2;
const BUILDERS : usize = 2;
const BOARD_SIZE : usize = 5;

#[derive(Debug, Clone)]
pub struct StandardBoard {
    pub slots : [Slot; 25],
    pub adjacencies : [[Slot ; 8] ; 25],
}

impl StandardBoard {
    pub fn slot_for(&self, position: Position) -> Option<Slot> {
        if position.x < 5 && position.y < 5 {
            Some(StandardBoard::slot(position))
        } else {
            None
        }
    }

    pub fn new() -> StandardBoard {
        let mut slots = [Slot(0) ; 25];
        let mut adjacencies = [[NONE ; 8] ; 25];

        for i in 0..25 {
            let slot = Slot(i as u8 );
            slots[i] = slot;
            let pos = StandardBoard::position(slot);
            // produce adjacencies based on position
            let x = pos.x as i8;
            let y = pos.y as i8;

            let mut j = 0;

            for nx in (x-1)..(x+2) {
                for ny in (y-1)..(y+2) {
                    let adjacent_position = Position { x: nx as u8, y : ny as u8 };
                    if !(nx == x && ny == y) && nx >= 0 && nx < (BOARD_SIZE as i8) && ny >= 0 && ny < (BOARD_SIZE as i8) {
                        adjacencies[i][j] = StandardBoard::slot(adjacent_position);
                        j += 1;
                    }
                }
            }
        }

        StandardBoard {
            slots: slots,
            adjacencies: adjacencies,
        }
    }

    pub fn next_moves(&self, state:&State, move_sink: &mut Vec<Move>) {
        let builders_to_move = state.builder_locations[state.to_move.0 as usize];
        let builders_to_place = builders_to_move.iter().any(|&pl| pl == UNPLACED_BUILDER );

        if builders_to_place {
            // 25 * 25 is 625 base
            for a in 0..25 {
                let slot_a = Slot(a);
                if state.collision.get(slot_a) == 0 {
                    for b in (a+1)..25 {
                        let slot_b = Slot(b);
                        if state.collision.get(slot_b) == 0 {
                            move_sink.push(Move::PlaceBuilders { a: slot_a, b:slot_b });
                        }
                    }    
                }
            }
        } else {
            // iterate both
            for &builder_location in builders_to_move.iter() {
                if Self::valid(builder_location) {
                    // attempt all moves with this guy
                    let current_height = state.buildings.get(builder_location);
                    for &move_to in self.adjacencies[builder_location.0 as usize].iter() {
                        if move_to == NONE { // we've reached end of adjacencies
                            break;
                        }
                        // no dome/person there, and height is at most 1 up
                        if state.collision.get(move_to) == 0 && state.buildings.get(move_to) <= current_height + 1 {
                            for &build_at in self.adjacencies[move_to.0 as usize].iter() {
                                if build_at == NONE {
                                    break;
                                }
                                if state.collision.get(build_at) == 0 || build_at == builder_location {
                                    move_sink.push(Move::Move { from: builder_location, to:move_to, build: build_at });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn position(slot: Slot) -> Position {
        // slot is 0 -> 24
        let x = slot.0 % 5;
        let y = slot.0 / 5;
        Position { x:x, y:y }
    }

    pub fn slot(position:Position) -> Slot {
        Slot(position.x + position.y * 5)
    }

    pub fn apply(&self, mve:Move, state:&State) -> State { // this implicitly destroys the state atm.
        match mve {
            Move::PlaceBuilders { a, b } => {
                let player_to_move = state.to_move;
                let mut new_state = state.clone();

                new_state.builder_locations[player_to_move.0 as usize][0] = a;
                new_state.collision = new_state.collision.set(a, 1);
                new_state.builder_locations[player_to_move.0 as usize][1] = b;
                new_state.collision = new_state.collision.set(b, 1);
                new_state.to_move = new_state.next_player();                

                new_state
            },
            Move::Move { from, to, build } => {
                let player_to_move = state.to_move;
                let mut new_state = state.clone();
                // update builder collision
                
                // assign updated builder location
                for i in 0..BUILDERS {
                    let builder_location = new_state.builder_locations[player_to_move.0 as usize][i];    
                    if builder_location == from {
                        new_state.collision = new_state.collision.set(from, 0);
                        new_state.collision = new_state.collision.set(to, 1);
                        new_state.builder_locations[player_to_move.0 as usize][i] = to; // place this builder
                        break;
                    }
                }
                // perform build
                new_state.build_at(build);
          
                // alternate player
                new_state.to_move = new_state.next_player();
                new_state
            },
        }
    }

    pub fn valid(slot:Slot) -> bool {
        slot.0 < 25
    }

    // use this to detect before applying move
    pub fn ascension_winning_move(&self, state:&State, mve: Move) -> bool {
          match mve {
            Move::PlaceBuilders { .. } => false,
            Move::Move { to, .. } => state.buildings.get(to) == 3,
        }
    }

    pub fn print(&self, state:&State) -> String {
        let mut out = String::new();

        out.push_str(&format!(" === To move {:?} === \n", state.to_move));
        let divider = "+---+---+---+---+---+\n";
        let empty =   "|   |   |   |   |   |\n";

        for x in 0..BOARD_SIZE {
            out.push_str(divider);
            let mut terrain : Vec<String> = Vec::new();
            let mut players : Vec<String> = Vec::new();

            for y in 0..BOARD_SIZE {
                let slot = Self::slot(Position { x: x as u8, y: y as u8});
                // terrain
                if state.domes.get(slot) > 0 {
                    terrain.push("D".into());
                } else if state.buildings.get(slot) > 0 {
                    terrain.push(state.buildings.get(slot).to_string());
                } else {
                    terrain.push(" ".into());
                }
                // players
                let mut found = false;
                for i in 0..PLAYERS {
                    let player_locations = state.builder_locations[i];
                    for &pl in player_locations.iter() {
                        if pl == slot {
                            players.push(format!("P{}",i));
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    players.push(" ".into());
                }
            }
            // write terrain
            let terrain : Vec<String> = terrain.iter().map(|t| t.pad_to_width_with_alignment(3, Alignment::Middle)).collect();
            out.push_str("|");
            out.push_str(&terrain.join("|"));
            out.push_str("|\n");
            let players : Vec<String> = players.iter().map(|t| t.pad_to_width_with_alignment(3, Alignment::Middle)).collect();
            out.push_str("|");
            out.push_str(&players.join("|"));
            out.push_str("|\n");
            out.push_str(empty);
        }
        out.push_str(divider);
        out
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Move {
    PlaceBuilders { a: Slot, b: Slot },
    Move { from: Slot, to:Slot, build: Slot },
}

impl Move {
    pub fn to_slots(&self) -> Vec<Vec<Slot>> {
        match self {
            &Move::PlaceBuilders { a, b } => vec![vec![a, b], vec![b, a]],
            &Move::Move { from, to, build } => vec![vec![from, to, build]],
        }
    }
}





