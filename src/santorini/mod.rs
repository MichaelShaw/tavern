
use tavern_core::{Slot, Player, Position, Packed};
use tavern_core::game::santorini::*;

use tavern_service::ai::*;


use jam::{Vec3, Vec3f, HashSet, InputState, Color};
use jam::color::*;
use jam::color;
use jam::render::*;

use howl::SoundEvent;
use cgmath::{Zero, Vector3};

use rand;
use rand::{Rng, XorShiftRng, SeedableRng};

pub fn unseeded_rng() -> XorShiftRng {
    let mut threaded_rng = rand::thread_rng();
    let random_seed = [threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32()];
    // let manual_seed = [1_u32, 2, 3, 4];
    rand::XorShiftRng::from_seed(random_seed)
}

pub struct SantoriniGame {
    pub game : CoreGame, // should the core game have AI state?
    pub tentative : TentativeGame,

    pub analysis: Option<StateAnalysis>,

    pub cpu_players : HashSet<Player>,
    pub current_move_positions : Vec<Slot>,
    pub mouse_over_slot : Option<Slot>,

    pub atlas: SantoriniAtlas,

    pub rand: XorShiftRng,

    pub ai_service : AIService,
}

const BOARD_OFFSET : Vec3 = Vector3 { x: 1.0, y: 0.0, z: 1.0 };
const BUILDING_PIXEL_OFFSETS : [u32; 4] = [0, 5, 10, 12];

const PLAYER_COLORS : [Color; 2] = [RED, YELLOW];

impl SantoriniGame {
    pub fn new() -> SantoriniGame {
        let mut rng = unseeded_rng();

        // let cpu_players = hashset![Player(rng.gen_range(0, 2))]; 
        let cpu_players = hashset![Player(0)]; 
        
        let core_game = CoreGame::new(StandardBoard::new(), State::initial());
        let tentative = core_game.tentative(&Vec::new(), None);

        let game = SantoriniGame {
            game: core_game,
            tentative: tentative,

            analysis: None,

            cpu_players: cpu_players,
            current_move_positions: vec![],
            mouse_over_slot: None,

            atlas: SantoriniAtlas::build(),

            rand: rng,

            ai_service: AIService::new(),
        };

        if game.cpu_players.contains(&game.game.state.player()) {
            game.ai_service.request_analysis(&game.game.state);    
        }

        game
    }

    pub fn update(&mut self, intersection: Option<Vec3>, input_state: &InputState, sound_event_sink: &mut Vec<SoundEvent>) {
    	if let Some(intersects_at) = intersection {
    		self.mouse_over_slot = Position::from(intersects_at.x - 1.0, intersects_at.z - 1.0).and_then(|p| self.game.board.slot_for(p));	
        } else {
            self.mouse_over_slot = None;
        }

        let mut tentative = self.game.tentative(&self.current_move_positions, self.mouse_over_slot);

		if self.cpu_players.contains(&self.game.state.player()) {
            if let Some(ref analysis) = self.analysis.clone() {
                if analysis.terminal {
                    if let Some(&(mve, h)) = analysis.moves.first() {
                        if self.game.next_moves.iter().any(|&m| m == mve) {
                            println!("playin move with heuristic {:?}", h);
                            self.play_move(mve);
                        }
                    }
                }
            }
        } else {
            if let Some(sl) = self.mouse_over_slot {
                if input_state.mouse.left_released() {
                    println!("pushing slot {:?}", sl);
                    if tentative.move_count > 0 {
                        self.current_move_positions.push(sl);
                        sound_event_sink.push(SoundEvent {
                            name: "place_tile".into(),
                            position: Vec3f::zero(),
                            gain: 1.0,
                            pitch: 1.0,
                            attenuation:1.0,
                            loop_sound: false,
                        });
                        println!("move is A-OK! matching moves -> {:?}", tentative.move_count);
                    } else {
                        println!("tentative move isnt valid");
                    }
                }
            }

            // right click pops move builder
            if input_state.mouse.right_released() && !self.current_move_positions.is_empty() {
                self.current_move_positions.pop();
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
            let completed_moves : Vec<_> = self.game.next_moves.iter().filter(|m| {
                &m.to_slots() == &self.current_move_positions
            }).cloned().collect();

            if let Some(mve) = completed_moves.first() {
                self.play_move(* mve);
            }

            tentative = self.game.tentative(&self.current_move_positions, self.mouse_over_slot);
        }

        self.tentative = tentative;

        'ai_loop: loop {
            match self.ai_service.receive.try_recv() {
                Ok(analysis) => {
                    if analysis.state == self.game.state {
                        self.analysis = Some(analysis);
                    } else {
                        println!("wrong state :-/")
                    }
                }
                Err(_) => {
                    break 'ai_loop;
                }
            }
        }
    }

    pub fn play_move(&mut self, mve: Move) -> MatchStatus {
        let status = self.game.make_move(mve);
        match status {
            MatchStatus::Won(player) => {
                println!("uhh player {:?} won", player);
                self.reset();
            },
            MatchStatus::ToMove(_) => (),
        }
        self.current_move_positions.clear();
        self.analysis = None;
        if self.cpu_players.contains(&self.game.state.player()) {
            self.ai_service.request_analysis(&self.game.state);    
        }
        status
    }

    pub fn reset(&mut self) {
        self.cpu_players = hashset![Player(self.rand.gen_range(0, 2))]; 
        self.game = CoreGame::new(StandardBoard::new(), State::initial());
        self.tentative = self.game.tentative(&Vec::new(), None);
    }

    pub fn render(&self, opaque: &mut GeometryTesselator, trans: &mut GeometryTesselator, units_per_point: f64) {
    	opaque.draw_floor_tile(&self.atlas.background, 0, 0.0, 0.0, 0.0, 0.0, false);

        let next_player_color = PLAYER_COLORS[self.game.state.player().0 as usize];

        // DRAW MOUSE OVER
        if let Some(slot) = self.mouse_over_slot {
            let position = StandardBoard::position(slot);
            let v = Vec3::new(position.x as f64, 0.0, position.y as f64) + BOARD_OFFSET;
            trans.color = color::WHITE.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
        }

        let draw_state : &State = &self.tentative.proposed_state;

        // DRAW BOARD CONTENTS
        for &slot in &self.game.board.slots {
            let pos = StandardBoard::position(slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;

            let building_height = draw_state.buildings.get(slot);
            let dome = draw_state.domes.get(slot) == 1;

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
        for (player_id, locations) in draw_state.builder_locations.iter().enumerate() {
            for &slot in locations {
                if slot != UNPLACED_BUILDER {
                    let pos = StandardBoard::position(slot);
                    let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                    let building_height = draw_state.buildings.get(slot);
                    let vert_offset = (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
                    opaque.draw_floor_tile_at(&self.atlas.players[player_id as usize], 0, v + Vec3::new(0.0, vert_offset, 0.0), 0.12, false );
                }
            }
        }

        for slot in &self.tentative.matching_slots {
            let pos = StandardBoard::position(*slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
            trans.color = next_player_color.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
        }
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