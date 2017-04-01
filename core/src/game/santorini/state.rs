
use game::santorini::*;
use game::*;


// pub const HEIGHT_BUILDER_ORDER : [usize; 4] = [2,3,1,0];
pub const HEIGHT_BUILDER_ORDER : [usize; 4] = [3,2,1,0];

pub const INITIAL_STATE : State =  State {
    builders: [PACKED1_EMPTY; 2],
    building_major: PACKED1_EMPTY,
    building_minor: PACKED1_EMPTY,
    domes: PACKED1_EMPTY,
    to_move: Player(0),
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct State {
    pub builders: [Packed1; 2],
    pub building_major : Packed1, 
    pub building_minor : Packed1,
    pub domes : Packed1,
    pub to_move : Player,
}

impl State {
    pub fn building_map(&self) -> [Packed1; 4] {
        [
            !self.building_major & !self.building_minor,
            !self.building_major & self.building_minor,
            self.building_major & !self.building_minor,
            self.building_major & self.building_minor,
        ]
    }
    pub fn current_builders(&self) -> Packed1 {
        self.builders[self.to_move.0 as usize]
    }

    pub fn collision(&self) -> Packed1 {
        self.builders[0] | self.builders[1] | self.domes
    }

    pub fn player(&self) -> Player {
        self.to_move
    }

    pub fn next_player(&self) -> Player {
        Player((self.to_move.0 + 1) % 2)
    }

    pub fn hash_height(&self, slot:Slot) -> usize { // should output 0-4
        if self.domes.get(slot) == 1 {
            4
        } else {
            self.get_building_height(slot) as usize
        }
    }

    pub fn without_builder_at(&self, slot:Slot) -> State {
        let mut new_state = self.clone();
        let mask = (1 << slot.0) ^ ALL_MASK_32;
        new_state.builders[0].0 &= mask;
        new_state.builders[1].0 &= mask;
        new_state
    }

    pub fn get_building_height(&self, slot:Slot) -> u8 {
        self.building_major.get(slot) * 2 + self.building_minor.get(slot)
    }

    pub fn set_building_height(&mut self, slot:Slot, height: u8) {
        self.building_major.set(slot, (height >> 1) & 1); // that is ... not at all write
        self.building_minor.set(slot, height & 1);
    }

    pub fn build_at(&mut self, slot:Slot) {
        let height = self.get_building_height(slot);
        let build_dome = height == 3;
        if build_dome {
            self.domes.toggle(slot);
            // self.collision = self.collision.set(slot, 1);
        } else {
            self.set_building_height(slot, height + 1);
        }
    }
}

