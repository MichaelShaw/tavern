
use tavern_core::{Slot, Player, Position, Packed};
use tavern_core::game::santorini;


use jam::{Vec3, Vec3f, HashSet, InputState, Color};
use jam::color::*;
use jam::color;
use jam::render::*;

use howl::SoundEvent;
use cgmath::{Zero, Vector3};


pub struct SantoriniGame {
    pub board: santorini::StandardBoard,
    // rest is per game, more transient
    pub state: santorini::State,
    pub cpu_players : HashSet<Player>,
    pub move_builder : MoveBuilder,
    pub mouse_over_slot : Option<Slot>,

    pub atlas: SantoriniAtlas,
}

const BOARD_OFFSET : Vec3 = Vector3 { x: 1.0, y: 0.0, z: 1.0 };
const BUILDING_PIXEL_OFFSETS : [u32; 4] = [0, 5, 10, 12];

const PLAYER_COLORS : [Color; 2] = [RED, YELLOW];

impl SantoriniGame {
    pub fn new() -> SantoriniGame {
        let cpu_players = HashSet::default();
        let state = santorini::State::initial();

        SantoriniGame {
            board: santorini::StandardBoard::new(),
            state: state,
            cpu_players: cpu_players,
            move_builder : MoveBuilder { positions: vec![] },
            mouse_over_slot: None,
            atlas: SantoriniAtlas::build(),
        }
    }

    pub fn update(&mut self, intersection: Option<Vec3>, input_state: &InputState, sound_event_sink: &mut Vec<SoundEvent>) {
    	if let Some(intersects_at) = intersection {
    		self.mouse_over_slot = Position::from(intersects_at.x - 1.0, intersects_at.z - 1.0).and_then(|p| self.board.slot_for(p));	

    		if self.cpu_players.contains(&self.state.player()) {
	            println!("waiting on a cpu");
	        } else {
	            let mut moves = Vec::new();
	            self.board.next_moves(&self.state, &mut moves);

	            // left click adds to move builder if legal
	            if let Some(sl) = self.mouse_over_slot {
	                if input_state.mouse.left_released() {
	                    println!("pushing slot {:?}", sl);
	                    self.move_builder.positions.push(sl);
	                    let matching_move_count = moves.iter().filter(|m| m.to_slots().iter().any(|sls| sls.starts_with(&self.move_builder.positions)) ).count();
	                    if matching_move_count == 0 {
	                        println!("no legal moves, popping!");
	                        self.move_builder.positions.pop();
	                    } else {
	                        sound_event_sink.push(SoundEvent {
	                            name: "place_tile".into(),
	                            position: Vec3f::zero(),
	                            gain: 1.0,
	                            pitch: 1.0,
	                            attenuation:1.0,
	                            loop_sound: false,
	                        });
	                        println!("move is A-OK! matching moves -> {:?}", matching_move_count);
	                    }
	                }
	            }

	            // right click pops move builder
	            if input_state.mouse.right_released() && !self.move_builder.positions.is_empty() {
	                self.move_builder.positions.pop();
	                sound_event_sink.push(SoundEvent {
	                    name: "select".into(),
	                    position: Vec3f::zero(),
	                    gain: 1.0,
	                    pitch: 1.0,
	                    attenuation:1.0,
	                    loop_sound: false,
	                });
	            }

	            // if we have a completd move, apply it to the board!
	            let completed_moves : Vec<_> = moves.iter().filter(|m| m.to_slots().iter().any(|sls| sls == &self.move_builder.positions)).collect();
	            if let Some(mve) = completed_moves.first() {
	                println!("we have a completed move -> {:?} applying it to state", mve);
	                self.state = self.board.apply(**mve, &self.state);
	                self.move_builder.positions.clear();
	            }
	        }
    	}
    }

    pub fn render(&self, opaque: &mut GeometryTesselator, trans: &mut GeometryTesselator, units_per_point: f64) {
    	opaque.draw_floor_tile(&self.atlas.background, 0, 0.0, 0.0, 0.0, 0.0, false);

        let next_player_color = PLAYER_COLORS[self.state.player().0 as usize];

        // DRAW MOUSE OVER
        if let Some(slot) = self.mouse_over_slot {
            let position = santorini::StandardBoard::position(slot);
            let v = Vec3::new(position.x as f64, 0.0, position.y as f64) + BOARD_OFFSET;
            trans.color = color::WHITE.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
        }

        // DRAW MOVE BUILDER
        for &slot in &self.move_builder.positions {
            let position = santorini::StandardBoard::position(slot);
            let v = Vec3::new(position.x as f64, 0.0, position.y as f64) + BOARD_OFFSET;
            trans.color = next_player_color.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
        }

        // DRAW BOARD CONTENTS
        for &slot in &self.board.slots {
            let pos = santorini::StandardBoard::position(slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;

            let building_height = self.state.buildings.get(slot);
            let dome = self.state.domes.get(slot) == 1;

            // RENDER THE BUILDING
            for i in 0..building_height {
                let vert_offset = (BUILDING_PIXEL_OFFSETS[i as usize] as f64) * units_per_point;
                opaque.draw_floor_tile_at(&self.atlas.buildings[i as usize], 0, v + Vec3::new(0.0, vert_offset, 0.0), 0.10, false)
            }
            // RENDER THE DOME
            if dome {
                let vert_offset = (BUILDING_PIXEL_OFFSETS[3] as f64) * units_per_point;
                opaque.draw_floor_tile_at(&self.atlas.dome, 0, v + Vec3::new(0.0, vert_offset, 0.0), 0.10, false)
            }
        }

        // DRAW THE GUYS
        for (player_id, locations) in self.state.builder_locations.iter().enumerate() {
            for &slot in locations {
                if slot != santorini::UNPLACED_BUILDER {
                    let pos = santorini::StandardBoard::position(slot);
                    let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                    let building_height = self.state.buildings.get(slot);
                    let vert_offset = (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
                    opaque.draw_floor_tile_at(&self.atlas.players[player_id as usize], 0, v + Vec3::new(0.0, vert_offset, 0.0), 0.12, false );
                }
            }
        }


        let mut moves = Vec::new();
        self.board.next_moves(&self.state, &mut moves);
        let legal_moves : Vec<_> = moves.iter().flat_map(|m| m.to_slots()).filter(|sl| {
            sl.starts_with(&self.move_builder.positions)
        }).collect();

        let mut valid_slots : HashSet<Slot> = HashSet::default();
        for m in &legal_moves {
            let next_slot_idx = self.move_builder.positions.len() as usize;
            if next_slot_idx < m.len() {
                valid_slots.insert(m[next_slot_idx]);
            }
        }
        for slot in valid_slots {
            let pos = santorini::StandardBoard::position(slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
            trans.color = next_player_color.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
        }
    }
}

pub struct MoveBuilder {
    pub positions : Vec<Slot>
}

impl MoveBuilder {
    pub fn new() -> MoveBuilder {
        MoveBuilder { positions: vec![] }
    }
}

#[derive(Debug)]
pub struct SantoriniAtlas {
    pub background : TextureRegion,
    pub buildings: [TextureRegion; 3],
    pub dome: TextureRegion,
    pub players : [TextureRegion; 2],
    pub indicator : TextureRegion,
}

impl SantoriniAtlas {
    pub fn build() -> SantoriniAtlas {
        let grid = TextureAtlas { texture_size: 512, tiles_wide: 32, tiles_high: 32 };

        SantoriniAtlas {
            background: grid.get(0, 0, 7, 8),
            buildings: [grid.at(7, 0), grid.at(7, 1), grid.at(7, 2)],
            dome: grid.at(7, 3),
            players: [grid.at(8, 0), grid.at(8, 1)],
            indicator: grid.at(9, 1),
        }
    }
}