
use tavern_core::{Slot, Player, Position, Packed};
use tavern_core::game::santorini::*;

use tavern_service::ai::*;

use cgmath::InnerSpace;
use jam::Vec2;

use aphid::HashSet;

use jam::BitmapFont;
use jam::{Vec3, Vec3f, InputState, Color, clamp};
use jam::color::*;
use jam::color;
use jam::Dimensions;
use jam::render::*;

use std::path::PathBuf;

use howl::SoundEvent;
use cgmath::{Zero, Vector3};

use aphid;

use rand;
use rand::{Rng, XorShiftRng, SeedableRng};

pub fn unseeded_rng() -> XorShiftRng {
    let mut threaded_rng = rand::thread_rng();
    let random_seed = [threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32()];
    // let manual_seed = [1_u32, 2, 3, 4];
    rand::XorShiftRng::from_seed(random_seed)
}

pub enum StateTransition {
    PlayerWin,
    PlayerLoss,
    NewInteractionState(InteractionState),
}

// stuff relevant to the current game in action
pub struct PlayerGame {
    pub interaction_state: InteractionState,
    pub board_state: BoardState,
    pub tentative: TentativeState,
    pub analysis: Option<StateAnalysis>,
    pub cpu_players : HashSet<Player>,
    pub current_move_positions : Vec<Slot>,
}

fn player_type_for(state:&State, cpu_players: &HashSet<Player>) -> PlayerType {
    if cpu_players.contains(&state.to_move) {
        PlayerType::AI
    }  else {
        PlayerType::Human
    }
}

impl InteractionState {
    pub fn awaiting_input(state:&State, cpu_players: &HashSet<Player>) -> InteractionState {
        InteractionState::AwaitingInput { player: state.to_move, player_type: player_type_for(state, cpu_players) }
    }
}

#[derive(Eq, Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Progress {
    pub level: usize,
    pub wins: usize,
}

pub fn wins_to_pass_for_level(level: usize) -> usize {
    match level {
        0...3 => 1,
        4...6 => 2,
        _ => 4,
    }
}

impl Progress {
    pub fn cpu_players(&self) -> HashSet<Player> {
        if self.wins % 2 == 0 {
            hashset![Player(1)]
        } else {
            hashset![Player(0)]
        }
    }

    pub fn ai_profile(&self) -> (AIProfile, Option<f64>) {
        (AIProfile {
            depth: self.level as Depth,
            heuristic: HeuristicName::AdjustedNeighbour,
        }, Some(15.0))
    }

    pub fn win(&mut self) {
        self.wins += 1;
        if self.wins == wins_to_pass_for_level(self.level) {
            self.level += 1;
            self.wins = 0;
        }
    }

    pub fn loss(&mut self) {
        if self.wins == 0 { // can't go lower than depth 2
            if self.level > 2 {
                self.level -= 1;
                self.wins = wins_to_pass_for_level(self.level) - 1;
            } 
        } else {
            self.wins -= 1;
        }
    }
}

pub const DEFAULT_PROGRESS : Progress = Progress { level: 2, wins: 0 };

pub struct SantoriniGame {
    pub game : PlayerGame, // should the core game have AI state?
    pub atlas: SantoriniAtlas,
    pub rand: XorShiftRng,
    pub ai_service : AIService,
    pub progress: Progress,
    pub profile_path: PathBuf,
}

const BOARD_OFFSET : Vec3 = Vector3 { x: 1.0, y: 0.0, z: 1.0 };
const BUILDING_PIXEL_OFFSETS : [u32; 4] = [0, 5, 10, 12];

const PLAYER_COLORS : [Color; 2] = [RED, YELLOW];

const VICTORY_WAIT : f64 = 5.0;
const ANIMATION_WAIT : f64 = 1.0;

impl SantoriniGame {
    pub fn new(profile_path: PathBuf) -> SantoriniGame {
        let progress : Progress = aphid::deserialize_from_json_file::<_,_,aphid::codec::JsonCodec>(&profile_path).ok().unwrap_or(DEFAULT_PROGRESS);

        println!("starting with progress -> {:?}", progress);

        let cpu_players = progress.cpu_players();
        
        let board_state = BoardState::new(StandardBoard::new(ZobristHash::new_unseeded()), INITIAL_STATE);
        let tentative = board_state.tentative(&Vec::new(), None);

        let player_game = PlayerGame {
            interaction_state: InteractionState::awaiting_input(&board_state.state, &cpu_players),
            board_state:board_state,
            tentative : tentative,
            analysis: None,
            cpu_players : cpu_players,
            current_move_positions : Vec::new(),
        };

        let ai_service = AIService::new();
        
        let game = SantoriniGame {
            game: player_game,
            mouse_over_slot: None,
            atlas: SantoriniAtlas::build(),
            rand: unseeded_rng(),
            ai_service: ai_service,
            progress:progress,
            profile_path: profile_path,
        };

        match &game.game.interaction_state {
            &InteractionState::AwaitingInput { player_type: PlayerType::AI, .. } => {
                game.requiest_ai_analysis();
            }
            _ => (),
        }

        game
    }

    pub fn update(&mut self, intersection: Option<Vec3>, input_state: &InputState, sound_event_sink: &mut Vec<SoundEvent>, delta_time: Seconds) {
        if let Some(intersects_at) = intersection {
            self.mouse_over_slot = Position::from(intersects_at.x - 1.0, intersects_at.z - 1.0).and_then(|p| self.game.board_state.board.slot_for(p));  
        } else {
            self.mouse_over_slot = None;
        }

        let mut new_interaction_state : Option<StateTransition> = None;

        match self.game.interaction_state {
            InteractionState::AwaitingInput { player_type: PlayerType::AI, .. } => {
                if let Some(ref analysis) = self.game.analysis.clone() {
                    if analysis.terminal {
                        println!("move analysis -> {:?}", analysis.best_move);
                        if let Some((mve, h)) = analysis.best_move {
                            if self.game.board_state.next_moves.iter().any(|&m| m == mve) {
                                println!("playin move with heuristic {:?} -> {:?}", h, mve);
                                self.play_move(mve);
                            }
                        }
                    }
                }
            },
            InteractionState::AwaitingInput { player_type: PlayerType::Human, .. } => {
                let tentative = self.game.board_state.tentative(&self.game.current_move_positions, self.mouse_over_slot);
                if let Some(sl) = self.mouse_over_slot {
                    if input_state.mouse.left_released() {
                        println!("pushing slot {:?}", sl);
                        if tentative.move_count > 0 {
                            self.game.current_move_positions.push(sl);
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
                if input_state.mouse.right_released() && !self.game.current_move_positions.is_empty() {
                    self.game.current_move_positions.pop();
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
                let completed_moves : Vec<_> = self.game.board_state.next_moves.iter().filter(|m| {
                    &m.to_slots() == &self.game.current_move_positions
                }).cloned().collect();

                if let Some(mve) = completed_moves.first() {
                    self.play_move(* mve);
                }
            },
            InteractionState::WaitingVictory { ref player, ref mut elapsed } => {
                *elapsed += delta_time;
                if *elapsed >= VICTORY_WAIT {
                    println!("we've waited long enough, reset");
                    if self.game.cpu_players.contains(player) {
                        new_interaction_state = Some(StateTransition::PlayerLoss);
                    } else {
                        new_interaction_state = Some(StateTransition::PlayerWin);
                    }
                }
            },
            InteractionState::AnimatingMove { ref mut elapsed, winner, .. } => {
                *elapsed = *elapsed + delta_time;
                if *elapsed >= ANIMATION_WAIT {
                    let is = if let Some(player) = winner {
                        InteractionState::WaitingVictory { player: player, elapsed: 0.0 }
                    } else {
                        InteractionState::awaiting_input(&self.game.board_state.state, &self.game.cpu_players)
                    };
                    new_interaction_state = Some(StateTransition::NewInteractionState(is));
                }
            },
        };
        match new_interaction_state {
            Some(StateTransition::NewInteractionState(is)) => {
                self.game.interaction_state = is
            },
            Some(StateTransition::PlayerLoss) => self.complete_game(false),
            Some(StateTransition::PlayerWin) => self.complete_game(true),
            None => (),
        }

        self.game.tentative = self.game.board_state.tentative(&self.game.current_move_positions, self.mouse_over_slot);


        'ai_loop: loop {
            match self.ai_service.receive.try_recv() {
                Ok(analysis) => {
                    if analysis.state == self.game.board_state.state {
                        self.game.analysis = Some(analysis);
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

    pub fn requiest_ai_analysis(&self) {
        let (ai_profile, time_limit) = self.progress.ai_profile();
        self.ai_service.request_analysis(self.game.board_state.state.clone(), ai_profile, time_limit);   
    }

    pub fn play_move(&mut self, mve: Move) -> MatchStatus {
        let prior_state = self.game.board_state.state.clone();

        let match_status = self.game.board_state.make_move(mve);
        let winning_player : Option<Player> = match match_status {
            MatchStatus::Won(player) => Some(player),
            MatchStatus::ToMove(_) => None,
        };

        self.game.interaction_state = InteractionState::AnimatingMove { 
            prior_state: prior_state.clone(), 
            mve:mve, 
            player_type: player_type_for(&prior_state, &self.game.cpu_players), 
            elapsed : 0.0, 
            winner: winning_player
        };

        // println!("played move new interaction state -> {:?}", self.game.interaction_state);
        self.game.current_move_positions.clear();
        self.game.analysis = None;
        if self.game.cpu_players.contains(&self.game.board_state.state.player()) {
            self.requiest_ai_analysis();
        }
        match_status
    }

    pub fn complete_game(&mut self, player_win: bool) { // RESET AINT GOOD ENOUGH ANYMORE
        println!("pre progress {:?}", self.progress);
        if player_win {
            println!("register win");
            self.progress.win();
        } else {
            println!("register loss");
            self.progress.loss();
        }
        println!("post progress {:?}", self.progress);
        let res = aphid::serialize_to_json_file::<_,_,aphid::codec::JsonCodec>(&self.progress, &self.profile_path);
        println!("serialization result -> {:?}", res);


        let cpu_players = self.progress.cpu_players();

        let board_state = BoardState::new(StandardBoard::new(ZobristHash::new_unseeded()), INITIAL_STATE);
        let tentative = board_state.tentative(&Vec::new(), None);
        self.game = PlayerGame {
            interaction_state: InteractionState::awaiting_input(&board_state.state, &cpu_players),
            board_state: board_state,
            tentative : tentative,
            analysis: None,
            cpu_players : cpu_players,
            current_move_positions : Vec::new(),
        };
        self.ai_service.reset();
        if self.game.cpu_players.contains(&self.game.board_state.state.player()) {
            self.requiest_ai_analysis();
        }
    }

    pub fn render_ui(&self, ui: &mut GeometryTesselator, font: &BitmapFont, font_layer: u32, dimensions: Dimensions) {
        let scale = 1.0 / dimensions.scale as f64;
        let scaled_font_size = (font.description.pixel_size as f64) * scale;

        ui.color = color::BLACK.float_raw();

        let (p_x, p_y) = dimensions.points();

        let progress_text = format!("Depth {}\nWins {}/{}", self.progress.level, self.progress.wins, wins_to_pass_for_level(self.progress.level));

        let at = Vec2::new(20.0,  p_y - scaled_font_size - 20.0);

        text::render_text(
            &progress_text, 
            font, 
            font_layer,
            at,
            -1.0, // i assume this is because our coordinate system is hosed ... 
            scale,
            ui, 
            Some(300.0)
        );

        let status : &str = match self.game.interaction_state {
            InteractionState::AwaitingInput { player_type: PlayerType::AI, .. } => "Waiting on AI Opponent ...",
            InteractionState::AwaitingInput { player_type: PlayerType::Human, .. } => "Your move.",
            InteractionState::WaitingVictory { player, .. } => {
                if self.game.cpu_players.contains(&player) {
                    "Defeat"
                } else {
                    "Victory!"
                }
            },
            InteractionState::AnimatingMove { .. } => "Moving ...",
        };

        let status_size = text::measure(status, font, scale, None);

        let status_at = Vec2::new(p_x / 2.0 - status_size.x / 2.0, 20.0);

        text::render_text(
            &status, 
            font, 
            font_layer,
            status_at,
            -1.0, // i assume this is because our coordinate system is hosed ... 
            scale,
            ui, 
            None
        );
    }

    pub fn render(&self, opaque: &mut GeometryTesselator, trans: &mut GeometryTesselator, units_per_point: f64) {
    	opaque.draw_floor_tile(&self.atlas.background, 0, 0.0, 0.0, 0.0, 0.0, false);

        let next_player_color = PLAYER_COLORS[self.game.board_state.state.player().0 as usize];

        match &self.game.interaction_state {
            &InteractionState::AnimatingMove { player_type: PlayerType::Human, .. } => {
                 self.draw_opaques(&self.game.board_state.state, opaque, units_per_point)
            }
            &InteractionState::AnimatingMove { ref prior_state, mve, player_type:PlayerType::AI, elapsed, .. } => {
                let subtracted_state = subtract(prior_state, mve);
                self.draw_opaques(&subtracted_state, opaque, units_per_point);

                let progress = clamp(elapsed / ANIMATION_WAIT, 0.0, 1.0);
                match mve {
                    Move::PlaceBuilders { a, b } => {
                        for slot in vec![a, b] {
                            let v = Self::exact_position(&subtracted_state, slot, units_per_point);
                            trans.color = [1.0, 1.0, 1.0, progress as f32];
                            trans.draw_floor_tile_at(&self.atlas.players[subtracted_state.to_move.0 as usize], 0, v, 0.15, false );
                        }
                    },
                    Move::Move { from, to, build } => {
                        let from_position = Self::exact_position(&subtracted_state, from, units_per_point);
                        let to_position = Self::exact_position(&subtracted_state, to, units_per_point);
                        let exact_position = from_position.lerp(to_position, progress);
                        trans.color = WHITE.float_raw();
                        trans.draw_floor_tile_at(&self.atlas.players[subtracted_state.to_move.0 as usize], 0, exact_position, 0.15, false );

                        // progress for fading in building
                        trans.color = [1.0, 1.0, 1.0, progress as f32];
                        let building_height = subtracted_state.get_building_height(build);
                        let is_dome = building_height == 3;

                        let pos = StandardBoard::position(build);
                        

                        if is_dome {
                            let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                            v.y += (BUILDING_PIXEL_OFFSETS[3] as f64) * units_per_point;
                            trans.draw_floor_tile_at(&self.atlas.dome, 0, v, 0.10, false)
                        } else {
                            let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                            v.y += (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
                            trans.draw_floor_tile_at(&self.atlas.buildings[building_height as usize], 0, v, 0.10, false)
                        }
                    },
                }
            },
            &InteractionState::AwaitingInput { player_type: PlayerType::AI, .. } => {
                self.draw_opaques(&self.game.board_state.state, opaque, units_per_point);
            },
            &InteractionState::AwaitingInput { player_type: PlayerType::Human, ..  } => {
                self.draw_opaques(&self.game.tentative.proposed_state, opaque, units_per_point);
                for slot in &self.game.tentative.matching_slots {
                    let pos = StandardBoard::position(*slot);
                    let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                    trans.color = next_player_color.float_raw();
                    trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.1, false);
                }
            },
            &InteractionState::WaitingVictory { .. } => {
                self.draw_opaques(&self.game.board_state.state, opaque, units_per_point);
            },
        }
        
        trans.color = WHITE.float_raw();

         // DRAW MOUSE OVER
        if let Some(slot) = self.mouse_over_slot {
            let position = StandardBoard::position(slot);
            let v = Vec3::new(position.x as f64, 0.0, position.y as f64) + BOARD_OFFSET;
            trans.color = color::WHITE.float_raw();
            trans.draw_floor_tile_at(&self.atlas.indicator, 0, v, 0.12, false);
        }
    }

    pub fn exact_position(state:&State, slot:Slot, units_per_point: f64) -> Vec3 {
        let pos = StandardBoard::position(slot);
        let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
        let building_height = state.get_building_height(slot);
        v.y += (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
        v
    }

    pub fn draw_opaques(&self, state: &State, opaque: &mut GeometryTesselator, units_per_point: f64) {
        // DRAW BOARD CONTENTS
        for &slot in &self.game.board_state.board.slots {
            let pos = StandardBoard::position(slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;

            let building_height = state.get_building_height(slot);
            let dome = state.domes.get(slot) == 1;

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
        for (player_id, builders) in state.builders.iter().enumerate() {
            for slot in builders.iter() {
                if slot != UNPLACED_BUILDER {
                    let v = Self::exact_position(state, slot, units_per_point);
                    opaque.draw_floor_tile_at(&self.atlas.players[player_id as usize], 0, v, 0.15, false );
                }
            }
        }
    }
}

pub fn subtract(state:&State, mve:Move) -> State {
    let mut new_state = state.clone();
    match mve {
        Move::PlaceBuilders { .. } => (),
        Move::Move { from, .. } => {
            new_state = new_state.without_builder_at(from);
        },
    }
    new_state
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