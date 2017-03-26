
use game::santorini::*;
// use time;
use game::*;

impl StandardBoard {
    pub fn perft(&self, state: &State, depth: usize, move_stack : &mut MoveStack) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut n = 0;

        let stack_begin = move_stack.next;
        self.next_moves(state, move_stack);
        let stack_end = move_stack.next;

        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];

            if self.ascension_winning_move(state, mve) {
                n += 1;
            } else {
                let new_state = self.apply(mve, state);
                n += self.perft(&new_state, depth - 1, move_stack);
            }
        }
        
        move_stack.next = stack_begin;

        n
    }

    pub fn new_apply(&self, mve:Move, state:&NewState) -> NewState {
        match mve {
            Move::PlaceBuilders { a, b } => {
                let player_to_move = state.to_move;

                let mut new_state = state.clone();

                let new_builder_mask = Packed1(1 << a.0 | 1 << b.0);

                new_state.builders[player_to_move.0 as usize] |= new_builder_mask;
                new_state.collision |= new_builder_mask;

                new_state.to_move = new_state.next_player();                

                new_state
            },
            Move::Move { from, to, build } => {
                let player_to_move = state.to_move;
                let mut new_state = state.clone();

                let movement_mask = Packed1(1 << from.0 | 1 << to.0);

                new_state.builders[player_to_move.0 as usize] ^= movement_mask;
                new_state.collision ^= movement_mask;
               
                new_state.build_at(build);
          
                new_state.to_move = new_state.next_player();
                new_state
            },
        }
    }

    pub fn transform_packed(transform: &SlotTransform, packed:Packed1) -> Packed1 {
        let mut out = PACKED1_EMPTY;

        for slot in packed.iter() {
            // println!("slot -> {:?}", slot);
            out.0 |= 1 << transform.slots[slot.0 as usize].0;
        }

        out
    }

    pub fn transform_packed2(transform: &SlotTransform, packed:[Packed1; 2]) -> [Packed1; 2] {
        let mut out = [PACKED1_EMPTY; 2];

        for i in 0..2 {
            for slot in packed[i as usize].iter() {
                // println!("slot -> {:?}", slot);
                out[i as usize].0 |= 1 << transform.slots[slot.0 as usize].0;
            }    
        }

        out
    }

    pub fn new_next_moves<T : MoveSink>(&self, state:&NewState, move_sink: &mut T) {
        let player_to_move = state.to_move;
        let builders = state.builders[player_to_move.0 as usize];
        let builders_to_place = builders.0 == 0;


        let collision = state.collision;

        if builders_to_place {
            let mut seen : HashSet<[Packed1; 2]> = HashSet::default();

            // place them
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
                            //  = StandardBoard::transform_packed(slot_transform, both_placed);

                            // println!("new testing -> {:?}", both_placed);

                            for slot_transform in &self.transforms {
                                
                                let transformed_builders = StandardBoard::transform_packed2(slot_transform, new_builders);

                                // let new_packed = 
                                // println!("out -> {:?}", new_packed);
                                if seen.contains(&transformed_builders) {
                                    // println!(" -> {:?} rejected due to {:?} ({:?} {:?})", both_placed, transformed_builders, Slot(a), Slot(b));
                                    dupe = true;
                                    break;
                                }
                            }

                            // PERFORM TRANSFORM/ROTATION HERE
                            if !dupe {
                                move_sink.sink(Move::PlaceBuilders { a: Slot(a), b:Slot(b) });    
                                seen.insert(new_builders);
                            }
                        }
                    }
                }
            }
        } else {
            for move_from in builders.iter() {
                let current_height = state.buildings.get(move_from);
                let moveable_adjacencies = self.packed_adjacencies[move_from.0 as usize] & (!collision);
                for move_to in moveable_adjacencies.iter() {
                    if state.buildings.get(move_to) <= current_height + 1 {
                        // add non collideables, then flip our original movement location
                        let buildable_adjacencies = self.packed_adjacencies[move_to.0 as usize] & (!collision) ^ Packed1(1 << move_from.0); // remove from
                        for build_at in buildable_adjacencies.iter() {
                            move_sink.sink(Move::Move { from: move_from, to:move_to, build: build_at });
                        }
                    }
                }
            }
        }
    }

    pub fn new_ascension_winning_move(&self, state:&NewState, mve: Move) -> bool {
        match mve {
            Move::PlaceBuilders { .. } => false,
            Move::Move { to, .. } => state.buildings.get(to) == 3,
        }
    }

    pub fn new_perft(&self, state: &NewState, depth: usize, move_stack : &mut MoveStack) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut n = 0;

        let stack_begin = move_stack.next;
        self.new_next_moves(state, move_stack);
        let stack_end = move_stack.next;

        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];

            if self.new_ascension_winning_move(state, mve) {
                n += 1;
            } else {
                let new_state = self.new_apply(mve, state);
                n += self.new_perft(&new_state, depth - 1, move_stack);
            }
        }

        move_stack.next = stack_begin;

        n
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct NewState {
    pub builders: [Packed1; 2],
    pub buildings : Packed2, 
    pub domes : Packed1,
    pub collision : Packed1,
    pub to_move : Player,
}

impl NewState {
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

pub const INITIAL_NEW_STATE : NewState =  NewState {
    builders: [PACKED1_EMPTY; 2],
    buildings: PACKED2_EMPTY,
    domes: PACKED1_EMPTY,
    collision: PACKED1_EMPTY,
    to_move: Player(0),
};

#[cfg(test)]
mod tests {
    // use game::santorini::*;
    use super::*;

    // #[test]
    fn compare_moves() {
        let board = StandardBoard::new(ZobristHash::new_unseeded());
        let mut a_moves = Vec::new();
        let a_state = INITIAL_STATE;
        let mut b_moves = Vec::new();
        let b_state = INITIAL_NEW_STATE;
        board.next_moves(&a_state, &mut a_moves);
        board.new_next_moves(&b_state, &mut b_moves);

        println!("===== OLD ({}) ==== ", a_moves.len());
        for mve in a_moves {
            // println!(" -> {:?}", mve);

        }
        println!("====== NEW ({}) =====", b_moves.len());
        for mve in b_moves {
            // println!(" -> {:?}", mve);

        }
    }

    #[test]
    fn test_perft() {
        let mut move_stack = MoveStack::new();

        let board = StandardBoard::new(ZobristHash::new_unseeded());

        

        let state = State::initial();
        let depth = 4;
        let start = time::precise_time_ns();
        let moves = board.perft(&state, depth, &mut move_stack);
        let duration = time::precise_time_ns() - start;
        let as_seconds = (duration as f64) / 1_000_000_000f64;
        
        let million_moves_per_second = moves as f64 / 1_000_000f64 / as_seconds;
        println!("=== OLD === perft {} moves ({:.2}M/second) in {:.2} seconds", moves, million_moves_per_second, as_seconds);
    }

    #[test]
    fn new_test_perft() {
        let board = StandardBoard::new(ZobristHash::new_unseeded());

        let mut move_stack = MoveStack::new();

        let state = INITIAL_NEW_STATE;
        let depth = 4;
        let start = time::precise_time_ns();
        let moves = board.new_perft(&state, depth, &mut move_stack);
        let duration = time::precise_time_ns() - start;
        let as_seconds = (duration as f64) / 1_000_000_000f64;
        
        let million_moves_per_second = moves as f64 / 1_000_000f64 / as_seconds;
        println!("=== NEW === perft {} moves ({:.2}M/second) in {:.2} seconds", moves, million_moves_per_second, as_seconds);
    }
}