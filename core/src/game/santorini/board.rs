
use pad::PadStr;
use game::santorini::*;

pub const SLOT_COUNT : usize = 25;

#[derive(Debug, Clone)]
pub struct StandardBoard {
    pub slots : [Slot; SLOT_COUNT],
    pub adjacencies : [[Slot ; 8] ; SLOT_COUNT],
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

    pub fn transform_state(&self, state: &State, slot_transform: &SlotTransform) -> State {
        let mut new_state = state.clone();

        for i in 0..25 {
            let from = Slot(i);
            let to = slot_transform.slots[i as usize];

            new_state.buildings = new_state.buildings.set(to, state.buildings.get(from));
            new_state.domes = new_state.domes.set(to, state.domes.get(from));
            new_state.collision = new_state.collision.set(to, state.collision.get(from));
        }

        for player_id in 0..2 {
            for builder_id in 0..2 {
                let builder_location = state.builder_locations[player_id][builder_id];
                if Self::valid(builder_location) {
                    new_state.builder_locations[player_id][builder_id] = slot_transform.slots[builder_location.0 as usize];
                }
            }
        }

        new_state.ensure_ordered(Player(0));
        new_state.ensure_ordered(Player(1));
      
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

        for i in 0..25 {
            let slot = Slot(i as i8);
            slots[i] = slot;
            let pos = StandardBoard::position(slot);
            // produce adjacencies based on position
            let x = pos.x;
            let y = pos.y;

            let mut j = 0;

            for nx in (x-1)..(x+2) {
                for ny in (y-1)..(y+2) {
                    let adjacent_position = Position { x: nx, y : ny };
                    if !(nx == x && ny == y) && nx >= 0 && nx < (BOARD_SIZE as i8) && ny >= 0 && ny < (BOARD_SIZE as i8) {
                        adjacencies[i][j] = StandardBoard::slot(adjacent_position);
                        j += 1;
                    }
                }
            }
        }

        let mut board = StandardBoard {
            slots: slots,
            adjacencies: adjacencies,
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
        let builders_to_move = state.builder_locations[state.to_move.0 as usize];
        let builders_to_place = builders_to_move.iter().any(|&pl| pl == UNPLACED_BUILDER );

        if builders_to_place {
            // 25 * 25 is 625 base
            let mut seen : HashSet<BuilderLocations> = HashSet::default();

            for a in 0..25 {
                let slot_a = Slot(a);
                if state.collision.get(slot_a) == 0 {
                    for b in (a+1)..25 {
                        let slot_b = Slot(b);
                        if state.collision.get(slot_b) == 0 {
                            let mut slots = state.builder_locations;
                            slots[state.to_move.0 as usize] = [slot_a, slot_b];

                            let mut dupe = false;

                            for slot_transform in &self.transforms {
                                let new_slots = StandardBoard::transform_slots(slot_transform, slots);
                                if seen.contains(&new_slots) {
                                    dupe = true;
                                    break;
                                }
                            }

                            if !dupe {
                                move_sink.sink(Move::PlaceBuilders { a: slot_a, b:slot_b });    
                                seen.insert(slots);
                            }
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
                                    move_sink.sink(Move::Move { from: builder_location, to:move_to, build: build_at });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn next_moves_for_player(&self, state:&State, move_sink: &mut Vec<Move>) {
        let builders_to_move = state.builder_locations[state.to_move.0 as usize];
        let builders_to_place = builders_to_move.iter().any(|&pl| pl == UNPLACED_BUILDER );

        if builders_to_place {
            // 25 * 25 is 625 base
            for a in 0..25 {
                let slot_a = Slot(a);
                if state.collision.get(slot_a) == 0 {
                    for b in 0..25 {
                        let slot_b = Slot(b);
                        if a != b && state.collision.get(slot_b) == 0 {
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

    pub fn hash(&self, state: &State) -> StateHash {
        let mut hash = self.hash.to_move[state.to_move.0 as usize];

        for i in 0..BUILDERS {
            for &bl in &state.builder_locations[i] {
                if Self::valid(bl) {
                    hash = hash + self.hash.builders[i as usize][bl.0 as usize];
                }
            }
        }

        for &sl in &self.slots {
            hash = hash + self.hash.buildings[sl.0 as usize][state.hash_height(sl)];
        }

        hash
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

                new_state.ensure_ordered(player_to_move);

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

                new_state.ensure_ordered(player_to_move);
               
                // perform build
                new_state.build_at(build);
          
                // alternate player
                new_state.to_move = new_state.next_player();
                new_state
            },
        }
    }

    pub fn delta_hash(&self, state: &State, mve:Move) -> StateHash { // this implicitly destroys the state atm.
        match mve {
            Move::PlaceBuilders { a, b } => {
               let to_move = state.to_move.0 as usize;
               self.hash.switch_move + self.hash.builders[to_move][a.0 as usize] + self.hash.builders[to_move][b.0 as usize]
            },
            Move::Move { from, to, build } => {
               let to_move = state.to_move.0 as usize;
               let original_height = state.hash_height(build);


               let build_at = build.0 as usize;

                self.hash.switch_move + self.hash.builders[to_move][from.0 as usize] + self.hash.builders[to_move][to.0 as usize] + 
                self.hash.buildings[build_at][original_height] + self.hash.buildings[build_at][original_height + 1]
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
            Move::Move { to, .. } => state.buildings.get(to) == 3,
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