// use std::fmt;

// extern crate pad;

use super::util::*;
use pad::{PadStr, Alignment};

// magics
pub const UNPLACED_BUILDER : Slot = Slot(255);
pub const DEAD_BUILDER : Slot = Slot(254);
pub const NONE : Slot = Slot(253);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct State {
    pub builder_locations: [[Slot; 2]; 2],
    pub buildings : Packed2, 
    pub domes : Packed1,
    pub collision : Packed1,
    pub to_move : Player,
}

impl State {
    pub fn initial() -> State {
        State {
            builder_locations: [[UNPLACED_BUILDER, UNPLACED_BUILDER], [UNPLACED_BUILDER, UNPLACED_BUILDER]],
            buildings: Packed2::empty(),
            domes: Packed1::empty(),
            collision: Packed1::empty(),
            to_move: Player(0),
        }
    }

    pub fn next_player(&self) -> Player {
        Player((self.to_move.0 + 1) % 2)
    }
}

const PLAYERS : usize = 2;
const BUILDERS : usize = 2;
const BOARD_SIZE : usize = 5;


// basically an immutable set of optimization/lookup tables
pub struct StandardBoard {
    pub slots : [Slot; 25],
    pub adjacencies : [[Slot ; 8] ; 25],
}

impl StandardBoard {
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

    pub fn next_moves(&self, state:State, move_sink: &mut Vec<Move>) {
        let builders_to_move = state.builder_locations[state.to_move.0 as usize];
        let should_place = builders_to_move.iter().any(|&pl| pl == UNPLACED_BUILDER );
        if should_place {
            for &builder_location in builders_to_move.iter() {
                if builder_location == UNPLACED_BUILDER {
                    for &slot in self.slots.iter() {
                        if state.collision.get(slot) == 0 {
                            move_sink.push(Move::PlaceBuilder { at: slot });
                        }
                    }
                    break; // only attempt to place the firs tone
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
                                if state.collision.get(build_at) == 0 {
                                    let build_dome = state.buildings.get(build_at) == 3;
                                    move_sink.push(Move::Move { from: builder_location, to:move_to, build: Build { at: build_at, dome: build_dome }});
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

    pub fn apply(&self, mve:Move, state:State) -> State {
        match mve {
            Move::PlaceBuilder { at } => {
                let mut new_state = state;
                let player_to_move = state.to_move;
                // assign updated builder location (could dry this)
                for i in 0..BUILDERS {
                    let builder_location = new_state.builder_locations[player_to_move.0 as usize][i];    
                    if builder_location == UNPLACED_BUILDER {
                        // place this builder
                        new_state.builder_locations[player_to_move.0 as usize][i] = at; 
                        new_state.collision = new_state.collision.set(at, 1);
                        break;
                    }
                }
                // alternate player
                new_state.to_move = new_state.next_player();
                new_state
            },
            Move::Move { from, to, build: Build { at, dome } } => {
                let mut new_state = state;
                // update builder collision
                let player_to_move = state.to_move;
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
                if dome {
                    new_state.domes = new_state.domes.set(at, 1);
                    new_state.collision = new_state.collision.set(at, 1);
                } else {
                    // println!("applyin gnew buildints")
                    new_state.buildings = new_state.buildings.set(at, new_state.buildings.get(at) + 1);
                }
                // alternate player
                new_state.to_move = new_state.next_player();
                new_state
            },
        }
    }

    pub fn valid(slot:Slot) -> bool {
        slot.0 < 25
    }

    pub fn ascension_winner(&self, state:State) -> Option<Player> {
        // ascension win
        for player_id in 0..PLAYERS {
            for &builder_location in state.builder_locations[player_id as usize].iter() {
                if Self::valid(builder_location) && state.buildings.get(builder_location) == 3 {
                    return Some(Player(player_id as u8));
                }
            }
        }

        None
    }

    pub fn print(&self, state:State) -> String {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Move {
    PlaceBuilder { at: Slot },
    Move { from: Slot, to:Slot, build: Build },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Build {
    pub at: Slot, 
    pub dome: bool,
}
