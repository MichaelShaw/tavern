
use game::santorini::*;
use time;
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


    pub fn new_next_moves<T : MoveSink>(&self, state:&NewState, move_sink: &mut T) {

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

        let mut move_stack = MoveStack::new();

        let board = StandardBoard::new(ZobristHash::new_unseeded());
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