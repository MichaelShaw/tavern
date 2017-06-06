
use pad::PadStr;
use game::santorini::*;
use game::*;

use std::cmp::min;

pub const SLOT_COUNT : usize = 25;

#[derive(Debug, Clone)]
pub struct StandardBoard {
    pub slots : [Slot; SLOT_COUNT],
    pub adjacencies : [[Slot ; 8] ; SLOT_COUNT],
    pub packed_adjacencies : [Packed1 ; SLOT_COUNT],
    pub transforms : [SlotTransform; 7],
    pub hash : ZobristHash,
}

pub trait MoveSink {
    fn sink(&mut self, mve:Move);
}

impl MoveSink for Vec<Move> {
    fn sink(&mut self, mve:Move) {
        self.push(mve);
    }
}



impl StandardBoard {
    pub fn transform<F>(&self, f: F) -> SlotTransform where F: Fn(Position) -> Position {
        let mut transform = SlotTransform { slots: [Slot(0); 25] };
        for x in 0..5 {
            for y in 0..5 {
                let position = Position{ x: x, y: y };
                let sl = StandardBoard::slot(position);
                let t_position = f(position);

                transform.slots[sl.0 as usize] = StandardBoard::slot(t_position);
            }
        }
        transform
    }

    pub fn transform_packed(transform: &SlotTransform, packed:Packed1) -> Packed1 {
        let mut out = PACKED1_EMPTY;

        for slot in packed.iter() {
            // println!("slot -> {:?}", slot);
            out.0 |= 1 << transform.slots[slot.0 as usize].0;
        }

        out
    }

    pub fn transform_state(&self, state: &State, slot_transform: &SlotTransform) -> State {
        let mut new_state = state.clone();

        for i in 0..25 {
            let from = Slot(i);
            let to = slot_transform.slots[i as usize];

            new_state.set_building_height(to, state.get_building_height(from));
            new_state.domes.set(to, state.domes.get(from));
        }

        new_state.builders[0] = StandardBoard::transform_packed(slot_transform, new_state.builders[0]);
        new_state.builders[1] = StandardBoard::transform_packed(slot_transform, new_state.builders[1]);

 
        new_state
    }

    pub fn slot_for(&self, position: Position) -> Option<Slot> {
        if position.x < 5 && position.y < 5 {
            Some(StandardBoard::slot(position))
        } else {
            None
        }
    }

    pub fn permute(&self, state: &State, sink: &mut Vec<State>) {
        for trans in &self.transforms {
            sink.push(self.transform_state(state, trans));
        }
    }

    pub fn new(hash:ZobristHash) -> StandardBoard {
        let mut slots = [Slot(0) ; 25];
        let mut adjacencies = [[NONE ; 8] ; 25];

        let mut packed_adjacencies = [PACKED1_EMPTY ; SLOT_COUNT];

        for i in 0..25 {
            let slot = Slot(i as i8);
            slots[i] = slot;
            let pos = StandardBoard::position(slot);
            // produce adjacencies based on position
            let x = pos.x;
            let y = pos.y;

            let mut j = 0;


            let mut packed = PACKED1_EMPTY;

            for nx in (x-1)..(x+2) {
                for ny in (y-1)..(y+2) {
                    let adjacent_position = Position { x: nx, y : ny };
                    if !(nx == x && ny == y) && nx >= 0 && nx < (BOARD_SIZE as i8) && ny >= 0 && ny < (BOARD_SIZE as i8) {
                        let slot = StandardBoard::slot(adjacent_position);
                        adjacencies[i][j] = slot;
                        j += 1;

                        packed.0 |= 1 << slot.0;
                    }
                }
            }

            packed_adjacencies[i] = packed;
        }

        let mut board = StandardBoard {
            slots: slots,
            adjacencies: adjacencies,
            packed_adjacencies: packed_adjacencies,
            transforms: [EMPTY_SLOT_TRANSFORM; 7],
            hash: hash,
        };

        board.transforms = [
            board.transform(rotate_90),
            board.transform(rotate_180),
            board.transform(rotate_270),
            board.transform(reflect_x),
            board.transform(reflect_y),
            board.transform(reflect_diag_a),
            board.transform(reflect_diag_b)
        ];

        board
    }

    pub fn transform_slots(transform: &SlotTransform, slots: BuilderLocations) -> BuilderLocations {
        let mut out : BuilderLocations = [[Slot(0); 2]; 2];

        for player_id in 0..2 {
            for i in 0..2 {
                let sl = slots[player_id][i];
                if Self::valid(sl) {
                    out[player_id][i] = transform.slots[sl.0 as usize];
                } else {
                    out[player_id][i] = sl;
                }
            }
        }

        out
    }

    pub fn next_moves<T : MoveSink>(&self, state:&State, move_sink: &mut T) {
        let player_to_move = state.to_move;
        let builders = state.builders[player_to_move.0 as usize];
        let in_placement_phase = builders.0 == 0;

        let collision = state.builders[0] | state.builders[1] | state.domes;
        let available = !collision;

        if in_placement_phase {
            let mut seen : HashSet<[Packed1; 2]> = HashSet::default();

            for a in 0..25 {
                let a_mask = 1 << a;
                if a_mask & collision.0 == 0 {
                    for b in (a+1)..25 {
                        let b_mask = 1 << b;
                        if b_mask & collision.0 == 0 {
                            let both_placed = Packed1(a_mask | b_mask);
                            let mut dupe = false;

                            let mut new_builders = state.builders;
                            new_builders[player_to_move.0 as usize] = both_placed;

                            for slot_transform in &self.transforms {
                                let transformed_builders = StandardBoard::transform_packed2(slot_transform, new_builders); // this is the only usage of Packed2
                                if seen.contains(&transformed_builders) {
                                    dupe = true;
                                    break;
                                }
                            }

                            if !dupe {
                                move_sink.sink(Move::PlaceBuilders { a: Slot(a), b:Slot(b) });    
                                seen.insert(new_builders);
                            }
                        }
                    }
                }
            }
        } else {
            let heights = state.building_map();

            let mut available_builders = [
                builders, // anyone can move to height 0
                builders, // anyone can move to height 1
                PACKED1_EMPTY,
                PACKED1_EMPTY,
            ];

            for bl in builders.iter() {
                let height = state.get_building_height(bl);
                let max_height = min(height + 1, 3);
                for h in 2..(max_height+1) {
                    available_builders[h as usize].0 |= 1 << bl.0;
                }
            }

            // these are really any mutually exclusive masks ..... as long as they're mutually exclusive we can do whatever

            // this is dual height based

           
            let mut h = 3_usize; 
            loop {
                let receive = heights[h] & available;

                if available_builders[h].any() {
                    for move_from in available_builders[h].iter() {
                        let move_tos = receive & self.packed_adjacencies[move_from.0 as usize];
                        if move_tos.any() {
                            for move_to in move_tos.iter() {
                                let buildable_adjacencies = self.packed_adjacencies[move_to.0 as usize] & available ^ Packed1(1 << move_from.0); 
                                // there's always buildabl adjacencies ... or you couldnt have moved
                                for &build_height in &HEIGHT_BUILDER_ORDER {
                                    let buildings_of_height = buildable_adjacencies & heights[build_height];
                                    if buildings_of_height.any() {
                                        for build_at in buildings_of_height.iter() {
                                            move_sink.sink(Move::Move { from: move_from, to:move_to, build: build_at });
                                        }    
                                    }
                                }
                            }    
                        }
                    }    
                }
                

                if h == 0 {
                    break;
                }

                h -= 1;
            }
        }
    }

    pub fn next_moves_for_player(&self, state:&State, move_sink: &mut Vec<Move>) {
        let player_to_move = state.to_move;
        let builders = state.builders[player_to_move.0 as usize];
        let in_placement_phase = builders.0 == 0;

        let collision = state.builders[0] | state.builders[1] | state.domes;
        let available = !collision;

        if in_placement_phase {
            for a in 0..25 {
                let a_mask = 1 << a;
                let slot_a = Slot(a);
                if a_mask & collision.0 == 0 {
                    for b in 0..25 {
                        let b_mask = 1 << b;
                        let slot_b = Slot(b);
                        if a != b && b_mask & collision.0 == 0 {
                             move_sink.push(Move::PlaceBuilders { a: slot_a, b:slot_b });    
                        }
                    }    
                }
            }
        } else {
            for move_from in builders.iter() {
                let current_height = state.get_building_height(move_from);
                let moveable_adjacencies = self.packed_adjacencies[move_from.0 as usize] & available;
                for move_to in moveable_adjacencies.iter() {
                    if state.get_building_height(move_to) <= current_height + 1 {
                        // add non collideables, then flip our original movement location
                        let buildable_adjacencies = self.packed_adjacencies[move_to.0 as usize] & available ^ Packed1(1 << move_from.0); // remove from
                        for build_at in buildable_adjacencies.iter() {
                            move_sink.sink(Move::Move { from: move_from, to:move_to, build: build_at });
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

    pub fn hash(&self, state: &State) -> StateHash {
        let mut hash = self.hash.to_move[state.to_move.0 as usize];

        for i in 0..PLAYERS {
            for bl in state.builders[i as usize].iter() {
                hash = hash ^ self.hash.builders[i as usize][bl.0 as usize];
            }
        }

        for &sl in &self.slots {
            hash = hash ^ self.hash.buildings[sl.0 as usize][state.hash_height(sl)];
        }

        hash
    }

    pub fn apply(&self, mve:Move, state:&State) -> State {
        match mve {
            Move::PlaceBuilders { a, b } => {
                let player_to_move = state.to_move;

                let mut new_state = state.clone();

                let new_builder_mask = Packed1(1 << a.0 | 1 << b.0);

                new_state.builders[player_to_move.0 as usize] |= new_builder_mask;
                // new_state.collision |= new_builder_mask;

                new_state.to_move = new_state.next_player();                

                new_state
            },
            Move::Move { from, to, build } => {
                let player_to_move = state.to_move;
                let mut new_state = state.clone();

                let movement_mask = Packed1(1 << from.0 | 1 << to.0);

                new_state.builders[player_to_move.0 as usize] ^= movement_mask;
                // new_state.collision ^= movement_mask;
               
                new_state.build_at(build);
          
                new_state.to_move = new_state.next_player();
                new_state
            },
        }
    }

    pub fn delta_hash(&self, state: &State, mve:Move) -> StateHash { // this implicitly destroys the state atm.
        match mve {
            Move::PlaceBuilders { a, b } => {
               let to_move = state.to_move.0 as usize;
               self.hash.switch_move ^ self.hash.builders[to_move][a.0 as usize] ^ self.hash.builders[to_move][b.0 as usize]
            },
            Move::Move { from, to, build } => {
                let to_move = state.to_move.0 as usize;
                let original_height = state.hash_height(build);

                let build_at = build.0 as usize;

                self.hash.switch_move ^ self.hash.builders[to_move][from.0 as usize] ^ self.hash.builders[to_move][to.0 as usize] ^ 
                self.hash.buildings[build_at][original_height] ^ self.hash.buildings[build_at][original_height + 1]
            },
        }
    }

    pub fn valid(slot:Slot) -> bool {
        slot.0 >= 0
    }

    // use this to detect before applying move
    pub fn ascension_winning_move(&self, state:&State, mve: Move) -> bool {
          match mve {
            Move::PlaceBuilders { .. } => false,
            Move::Move { to, .. } => state.get_building_height(to) == 3,
        }
    }

    pub fn print(&self, state:&State) -> String {
        let mut out = String::new();

        out.push_str(&format!(" === To move {:?} === \n", state.to_move));
        let divider = "+---+---+---+---+---+\n";
        let empty =   "|   |   |   |   |   |\n";

        for y in 0..BOARD_SIZE {
            out.push_str(divider);
            let mut terrain : Vec<String> = Vec::new();
            let mut players : Vec<String> = Vec::new();

            for x in 0..BOARD_SIZE {
                let slot = Self::slot(Position { x: x as i8 , y: y as i8 });
                // terrain
                if state.domes.get(slot) > 0 {
                    terrain.push("D".into());
                } else if state.get_building_height(slot) > 0 {
                    terrain.push(state.get_building_height(slot).to_string());
                } else {
                    terrain.push(" ".into());
                }
                // players
                let mut found = false;
                for i in 0..PLAYERS {
                    for pl in state.builders[i as usize].iter() {
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