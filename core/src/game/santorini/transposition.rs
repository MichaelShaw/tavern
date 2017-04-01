use game::santorini::*;

use std::mem;

#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub enum EntryType {
    Exact,
    Lower,
    Upper,
}

#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub struct TranspositionEntry {
    // pub state: State,
    pub hash: StateHash, // 8 bytes
    pub value: HeuristicValue, // 2 bytes
    pub entry_type: EntryType, // 1 byte
    pub depth: i8, // 1 byte
    pub generation: Generation, // 1 byte
    pub best_move: Option<Move>, // 5 bytes
    
    // move in theory could be reduced to 2 bytes + flag -> 3 bytes  + optionality == 3-4 ish bytes ...
}

impl TranspositionEntry {
    pub fn value(&self, current_generation: Generation) -> i8 { // i feel this numerical type isn't correct
        self.depth - (current_generation - self.generation) as i8 * 2
    }
}

pub const NULL_ENTRY : TranspositionEntry = TranspositionEntry {
    // state: INITIAL_STATE,
    hash: StateHash(0),
    value: 0,
    entry_type: EntryType::Lower,
    depth: 0, 
    generation: 0,
    best_move: None,
};


#[derive(Eq, Copy, PartialEq, Clone, Debug)]
pub struct StateHash(pub u64);

pub const BUCKET_SIZE : usize = 4; // 24 x 4 = 96 bytes .... that's 3 cache lines ... my cache alignment sucks, i need to get to 16 bytes per entry
pub const STATE_HASH_ZERO : StateHash = StateHash(0);

use std::ops::BitXor;

pub type Generation = u8;

impl BitXor for StateHash {
    type Output = StateHash;

    fn bitxor(self, other: StateHash) -> StateHash {
        StateHash(self.0 ^ other.0)
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    pub generation : Generation,
    pub bucket_mask: u64,
    pub entries : Vec<TranspositionEntry>,
}

impl TranspositionTable {
    #[inline]
    pub fn bucket_location_for(&self, hash:StateHash) -> usize {
        ((hash.0 & self.bucket_mask) as usize) * BUCKET_SIZE
    }

    pub fn increment_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1)
    }

    pub fn probe(&self, hash:StateHash) -> (usize, bool) { // entry_location, found/match
        let bucket_location = self.bucket_location_for(hash);
        
        // look for null or same position
        for i in 0..BUCKET_SIZE {
            let entry_location = bucket_location + i;
            let entry = &self.entries[entry_location];
            let same_hash = entry.hash == hash;
            if entry.hash.0 == 0 || same_hash {
                return (entry_location, same_hash)
            }
        }

        // ok nothing the same, let's replace the least valuable entry
        let current_generation = self.generation;

        let mut replace_idx : usize = bucket_location;
        for i in 0..BUCKET_SIZE {
            let entry_location = bucket_location + i;
            if self.entries[replace_idx].value(current_generation) > self.entries[entry_location].value(current_generation) { // if current replacement slot is newer than existing
                replace_idx = entry_location;
            }
        }

        return (replace_idx, false)
    }

    pub fn store(&mut self, idx: usize, hash:StateHash, value:HeuristicValue, depth: i8, entry_type: EntryType, best_move: Option<Move>) {
        let entry = &mut self.entries[idx];

        if entry.hash != hash || depth > (entry.depth - 4) || entry_type == EntryType::Exact {
            entry.hash = hash;
            entry.value = value;
            entry.entry_type = entry_type;
            entry.depth = depth;
            entry.generation = self.generation;
            entry.best_move = best_move;
        }
    }

    pub fn size_bytes(&self) -> usize {
        TranspositionTable::approx_size_bytes(self.entries.capacity())
    }

    pub fn approx_size_bytes(entry_count: usize) -> usize {
        mem::size_of::<TranspositionEntry>() * entry_count
    }

    pub fn reset(&mut self) {
        for i in 0..self.entries.len() {
            self.entries[i] = NULL_ENTRY;
        }
    }

    pub fn new(power_of_two:usize) -> TranspositionTable {
        let size = 1 << power_of_two;
        let mut bucket_mask : usize = 1;
        for _ in 0..(power_of_two-1-2) { // -1 is normal, the -2 is for buckets
            bucket_mask = bucket_mask | (bucket_mask << 1);
        }

        let mut entries = Vec::with_capacity(size);
        entries.resize(size, NULL_ENTRY);

        TranspositionTable {
            generation: 0,
            bucket_mask: bucket_mask as u64,
            entries: entries,
        }
    }
}



#[derive(Debug, Clone)]
pub struct ZobristHash {
    pub to_move : [StateHash; PLAYERS],
    pub switch_move : StateHash,
    pub builders : [[StateHash; SLOT_COUNT]; 2],
    pub buildings : [[StateHash; 5]; SLOT_COUNT], // 0 is 0 (no flip) to remove branching
}

use rand::Rng;
use rand::{XorShiftRng, ChaChaRng};

impl ZobristHash {
    pub fn new_unseeded() -> ZobristHash {
        Self::new(&mut XorShiftRng::new_unseeded())
    }
    
    pub fn new_unseeded_secure() -> ZobristHash {
        Self::new(&mut ChaChaRng::new_unseeded())
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

        hash.switch_move = hash.to_move[0] ^ hash.to_move[1];

        for builder_hashes in &mut hash.builders {
            // for i in 1..5 { // leave first one null
            //     builder_hashes[i] = StateHash(r.next_u64());
            // }
            // for i in 1..5 { // leave first one null
            //     builder_hashes[i] = StateHash(r.next_u64());
            // }
            for i in builder_hashes {
                *i = StateHash(r.next_u64());
            }
        }

        for building in &mut hash.buildings {
            for height in building {
                *height = StateHash(r.next_u64());    
            }
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use game::santorini::*;
    use std::mem;
    use super::*;

    #[test]
    fn table() {
        let mut table = TranspositionTable::new(5); // 8 boxes in theory
        println!("ok we have a table, entry count -> {:?}, mask -> {:b}", table.entries.len(), table.bucket_mask);

        


        for i in 1..40 {
            let hash = StateHash(i);
            let (idx, found) = table.probe(hash);
            println!("state {:?} -> idx {:?} found {:?}", i, idx, found);

            table.store(idx, hash, 12, 4, EntryType::Exact, None);
        }

    }

    #[test]
    fn my_zobist() {
        use super::Move::*;

        let hsh = ZobristHash::new_unseeded();
        println!("hash -> {:?}", hsh);
        let board = StandardBoard::new(hsh);

        let state = INITIAL_STATE;

        let a_state = board.apply(PlaceBuilders { a: Slot(0), b: Slot(6)}, &state);
        let a_hash = board.hash(&a_state);
        let b_state = board.apply(PlaceBuilders { a: Slot(0), b: Slot(7)}, &state);
        let b_hash = board.hash(&b_state);

        println!("a {:?} b {:?}", a_hash, b_hash);
    }

    #[test]
    fn sizes() {
        println!("State size -> {}", mem::size_of::<State>());
        println!("Move size -> {}", mem::size_of::<Move>());
        println!("Option<Move> size -> {}", mem::size_of::<Option<Move>>());
        println!("EntryType size -> {}", mem::size_of::<EntryType>());
        println!("TranspositionEntry size -> {}", mem::size_of::<TranspositionEntry>());

        // println!("talble entry count -> {}", TABLE_ENTRY_COUNT);
        println!("size of table -> {}", mem::size_of::<TranspositionTable>());


        println!("size of unit -> {}", mem::size_of::<()>())
    }

    pub const MAH_CAP : usize = 200_000_000;

    #[test]
    fn hash() {
        let new_hash = ZobristHash::new_unseeded();
        println!("constructing table");
        let table = TranspositionTable::new(26);
        println!("mask -> {:#b}", table.bucket_mask);
        println!("capacity -> {}", table.entries.capacity());
        println!("approx size -> {}", table.size_bytes());
        // println!("new hash -> {:?}", new_hash);

        use time;
        
    

        let start = time::precise_time_ns();
        let mut growable : Vec<TranspositionEntry> = Vec::with_capacity(MAH_CAP);
        // unsafe { growable.set_len(MAH_CAP) };
        for _ in 0..MAH_CAP {
            growable.push(NULL_ENTRY);
            // growable[i] = NULL_ENTRY;
        }
        println!("what is stupid -> {:?}", growable[3000]);
        let duration = (time::precise_time_ns() - start) as f64 / 1_000_000_000f64;
        println!("vec n set {:.3}", duration);
    
        // let huge_table = 
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
        let state = INITIAL_STATE;
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
            new_hash = new_hash ^ board.delta_hash(&new_state, mve);
            new_state = board.apply(mve, &new_state);
            println!("post {:?} hash is {:?}", mve, new_hash );
        }

        (new_state, new_hash)
    }
}
