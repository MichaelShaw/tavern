use game::santorini::*;



#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub enum EntryType {
    Exact,
    Lower,
    Upper,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TranspositionEntry {
    pub state: State,
    pub value: HeuristicValue,
    pub entry_type: EntryType,
    pub depth: u8,
    pub best_move: Option<Move>,
    // best move
}


#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub struct StateHash(pub u64);

pub const STATE_HASH_ZERO : StateHash = StateHash(0);

use std::ops::Add;

impl Add for StateHash {
    type Output = StateHash;

    fn add(self, other: StateHash) -> StateHash {
        StateHash(self.0 ^ other.0)
    }
}

pub const TABLE_ENTRY_COUNT : usize = 1 << 24; // 800MB
pub const TABLE_ENTRY_MASK : usize = 0b1111_1111_1111_1111_1111_1111_usize; // 24 bit mask

pub struct TranspositionTable {
    pub entries : [TranspositionEntry; TABLE_ENTRY_COUNT],
}

#[derive(Debug, Clone)]
pub struct ZobristHash {
    pub to_move : [StateHash; PLAYERS],
    pub switch_move : StateHash,
    pub builders : [[StateHash; SLOT_COUNT]; 2],
    pub buildings : [[StateHash; 5]; SLOT_COUNT], // 0 is 0 (no flip) to remove branching
}

use rand::Rng;
use rand::XorShiftRng;

impl ZobristHash {
    pub fn new_unseeded() -> ZobristHash {
        Self::new(&mut XorShiftRng::new_unseeded())
    }

    pub fn new<R : Rng>(r: &mut R) -> ZobristHash {
        let mut hash = ZobristHash {
            to_move: [STATE_HASH_ZERO; PLAYERS],
            switch_move : STATE_HASH_ZERO,
            builders : [[STATE_HASH_ZERO; SLOT_COUNT]; 2],
            buildings : [[STATE_HASH_ZERO; 5]; SLOT_COUNT],
        };

        for to_move in &mut hash.to_move {
            *to_move = StateHash(r.next_u64());
        }

        for builder_hashes in &mut hash.builders {
            for i in 1..5 { // leave first one null
                builder_hashes[i] = StateHash(r.next_u64());
            }
        }

        for building in &mut hash.buildings {
            for height in building {
                *height = StateHash(r.next_u64());    
            }
        }

        hash.switch_move = hash.to_move[0] + hash.to_move[1];

        hash
    }
}


#[cfg(test)]
mod tests {
    use game::santorini::*;
    use std::mem;
    use super::*;

    #[test]
    fn sizes() {
        println!("State size -> {}", mem::size_of::<State>());
        println!("Move size -> {}", mem::size_of::<Move>());
        println!("EntryType size -> {}", mem::size_of::<EntryType>());
        println!("TranspositionEntry size -> {}", mem::size_of::<TranspositionEntry>());

        println!("talble entry count -> {}", TABLE_ENTRY_COUNT);
        println!("size of table -> {}", mem::size_of::<TranspositionTable>());
    }

    #[test]
    fn hash() {
        let new_hash = ZobristHash::new_unseeded();
        println!("new hash -> {:?}", new_hash);

    }

    #[test] 
    fn hasher() {
        use super::Move::*;

        let moves_a : Vec<super::Move> = vec![
            PlaceBuilders { a: Slot(0), b: Slot(1)},
            PlaceBuilders { a: Slot(23), b: Slot(24)},
            Move { from: Slot(0), to: Slot(5), build: Slot(10) },
            Move { from: Slot(24), to: Slot(19), build: Slot(14) },
            Move { from: Slot(1), to: Slot(6), build: Slot(11) },
            Move { from: Slot(23), to: Slot(18), build: Slot(13) },
        ];

        let moves_b : Vec<super::Move> = vec![
            PlaceBuilders { a: Slot(0), b: Slot(1)},
            PlaceBuilders { a: Slot(23), b: Slot(24)},
            Move { from: Slot(1), to: Slot(6), build: Slot(11) },
            Move { from: Slot(23), to: Slot(18), build: Slot(13) },
            Move { from: Slot(0), to: Slot(5), build: Slot(10) },
            Move { from: Slot(24), to: Slot(19), build: Slot(14) },
        ];

        let board = StandardBoard::new(ZobristHash::new_unseeded());
        let state = State::initial();
        let init_hash = board.hash(&state);

        let playout_a = play_moves(&board, &state, &moves_a);
        let a_hash = board.hash(&playout_a);
        let playout_b = play_moves(&board, &state, &moves_b);
        let b_hash = board.hash(&playout_b);
       
        assert_eq!(playout_b, playout_b);
        assert_eq!(a_hash, b_hash);
        println!("they ok!");

        let (playout_c, c_hash) = play_moves_delta(&board, &state, &moves_a, init_hash);
        let (playout_d, d_hash) = play_moves_delta(&board, &state, &moves_b, init_hash);

        assert_eq!(playout_a, playout_c);
        assert_eq!(playout_c, playout_d);

        assert_eq!(a_hash, c_hash);
        assert_eq!(c_hash, d_hash);
        
        
        // let 
    }

    pub fn play_moves(board: &StandardBoard, state: &State, moves: &Vec<super::Move>) -> State {
        let mut new_state = state.clone();
        for &mve in moves {
            new_state = board.apply(mve, &new_state);
        }
        new_state
    }
// apply_hash(&self, state: &State, mve:Move)
    pub fn play_moves_delta(board: &StandardBoard, state: &State, moves: &Vec<super::Move>, hash:StateHash) -> (State, StateHash) {
        println!("playing out -> {:?}", moves);
        let mut new_state = state.clone();
        let mut new_hash = hash;
        for &mve in moves {
            new_hash = new_hash + board.delta_hash(&new_state, mve);
            new_state = board.apply(mve, &new_state);
            println!("post {:?} hash is {:?}", mve, new_hash );
        }

        (new_state, new_hash)
    }
}
