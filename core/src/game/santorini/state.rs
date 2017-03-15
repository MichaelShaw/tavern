
use game::santorini::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct State {
    pub builder_locations: BuilderLocations,
    pub buildings : Packed2, 
    pub domes : Packed1,
    pub collision : Packed1,
    pub to_move : Player,
}

impl State {
    pub fn hash_height(&self, slot:Slot) -> usize { // should output 0-4
        if self.domes.get(slot) == 1 {
            4
        } else {
            self.buildings.get(slot) as usize
        }
    }

    pub fn without_builder_at(&self, slot:Slot) -> State {
        let mut new_state = self.clone();
        for player_id in 0..2 {
            for builder_id in 0..2 {
                if new_state.builder_locations[player_id][builder_id] == slot {
                    new_state.builder_locations[player_id][builder_id] = UNPLACED_BUILDER;
                }
            }
        }
        new_state
    }

    pub fn is_ordered(&self) -> bool {
        self.builder_locations[0][0] <= self.builder_locations[0][1] && self.builder_locations[1][0] <= self.builder_locations[1][1]
    }

    pub fn ensure_ordered(&mut self, player:Player) {
        let builder_locations = &mut self.builder_locations[player.0 as usize];
        if builder_locations[1] < builder_locations[0] {
            builder_locations.swap(0, 1);
        }
     }       

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