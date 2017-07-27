


use std::path::PathBuf;

use rand;
use rand::{Rng, XorShiftRng, SeedableRng};

use cgmath::InnerSpace;
use cgmath::{Zero, Vector3};

use aphid;
use aphid::{Seconds, HashSet, Milliseconds};

use jam::BitmapFont;
use jam::{Vec2,Vec3, Vec3f, InputState, Color, clamp};
use jam::color::*;
use jam::color;
use jam::Dimensions;
use jam::render::*;

use howl::SoundEvent;

use psyk::game::{Player, Human};

use tavern_core;
use tavern_core::{Slot, Position, Packed};
use tavern_core::game::santorini::*;

use tavern_service::ai::*;
use tavern_service::game::*;
use tavern_service::tentative::*;
use tavern_service::board_state::*;
use tavern_service::event::*;


type PlayerSlot = tavern_core::Player;

pub struct SantoriniClient {
    pub profile : PlayerProfile,
    pub profile_path: PathBuf,

    pub game : ClientGame, // should the core game have AI state?
    pub board : StandardBoard,

    pub rand: XorShiftRng, // unsure why we need this .... for new games I guess ..... for now
    pub ai_service : AIService,

    pub atlas: SantoriniAtlas,  

    pub sound_events: Vec<SoundEvent>,  
}

 // should we move this to aphid
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




pub const DEFAULT_PROGRESS : Progress = Progress { level: 2, wins: 0 };


const BOARD_OFFSET : Vec3 = Vector3 { x: 1.0, y: 0.0, z: 1.0 };
const BUILDING_PIXEL_OFFSETS : [u32; 4] = [0, 5, 10, 12];

const PLAYER_COLORS : [Color; 2] = [RED, YELLOW];

const VICTORY_WAIT : Milliseconds = 5_000;
const ANIMATION_WAIT : Milliseconds = 1_000;

fn game_for(standard_board: &StandardBoard, players:Players) -> ClientGame {
    let board_with_moves = BoardWithMoves::for_board(BoardState::new(INITIAL_STATE), &standard_board);
    let interaction_state = InteractionState::awaiting_input(board_with_moves.state(), &players);
    ClientGame {
        board: board_with_moves, // board
        tentative : None, 
        players : players, 
        interactivity: interaction_state, // interactivity
        analysis : None, 
    }
}

impl SantoriniClient {
    pub fn new_profile<R : Rng>(rng: &mut R) -> PlayerProfile {
        PlayerProfile {
            player: Human { id: 12, name: "Steve".into() },
            progress: DEFAULT_PROGRESS,
        }
    }

    pub fn new(profile_path: PathBuf) -> SantoriniClient {
        let standard_board = StandardBoard::new(ZobristHash::new_unseeded());
        let mut rng = unseeded_rng();

        let profile : PlayerProfile = aphid::deserialize_from_json_file::<_,_,aphid::codec::JsonCodec>(&profile_path).ok().unwrap_or(SantoriniClient::new_profile(&mut rng));
        println!("starting with profile -> {:?}", profile);

        let game_players = profile.progress.players(&profile.player);

        let game = game_for(&standard_board, game_players);
        
        let ai_service = AIService::new();
        
        let client = SantoriniClient {
            profile : profile,
            profile_path: profile_path,

            game : game, // should the core game have AI state?
            board : standard_board,

            rand: rng, // unsure why we need this .... for new games I guess ..... for now
            ai_service : ai_service,

            atlas: SantoriniAtlas::build(), 

            sound_events: Vec::new(),
        };

        if client.game.waiting_on_ai() {
            client.request_ai_analysis();
        }

        client
    }

    pub fn process_multiple(&mut self, events: Vec<ClientLocalEvent>) {
        for ev in events {
            self.process(ev);
        }
    }

    pub fn process(&mut self, event: ClientLocalEvent) {
        use self::ClientLocalEvent::*;

        let mut events: Vec<ClientLocalEvent> = Vec::new();
        events.push(event);

        while let Some(ev) = events.pop() {
            println!("processing -> {:?}", ev);
            match ev {
                UpdateTentativeSlot(slot) => {
                    if let Some(ui_state) = self.game.players.mut_human_ui_state(&self.profile.player) {
                        ui_state.tentative_slot = slot;
                    }
                },
                PushCurrentSlot(slot) => {
                    if let Some(ui) = self.game.players.mut_human_ui_state(&self.profile.player) {
                        ui.current_slots.push(slot);

                        self.sound_events.push(SoundEvent {
                            name: "place_tile".into(),
                            position: [0.0; 3],
                            gain: 1.0,
                            pitch: 1.0,
                            attenuation:1.0,
                            loop_sound: false,
                        });

                        // if we have a completd move, apply it to the board!
                        let completed_moves : Vec<_> = self.game.board.legal_moves().iter().filter(|m| {
                            &m.to_slots() == &ui.current_slots
                        }).cloned().collect();

                        if let Some(mve) = completed_moves.first() {
                            events.push(PlayMove(*mve, None));
                        } 
                    }
                },
                PopCurrentSlot => {
                    let update = if let Some(ui) = self.game.players.mut_human_ui_state(&self.profile.player) {
                        ui.current_slots.pop();
                        true
                    } else {
                        false
                    };
                    if update {
                        self.sound_events.push(SoundEvent {
                             name: "select".into(),
                             position: [0.0; 3],
                             gain: 1.0,
                             pitch: 1.0,
                             attenuation:1.0,
                             loop_sound: false,
                        });
                    }
                },
                PlayMove(mve, h) => {
                    self.play_move(mve);
                },
                NewInteractionState(is) => self.game.interactivity = is,
                PlayerLoss => self.complete_game(false),
                PlayerWin => self.complete_game(true),
                NewAnalysis(analysis) => {
                    if &analysis.state == self.game.board.state() {
                        self.game.analysis = Some(analysis);
                    } else {
                        println!("wrong state :-/")
                    }
                },
            }
        }

        self.recalculate_tentative();
    }

    pub fn recalculate_tentative(&mut self) {
        self.game.tentative = if let Some(ui) = self.game.players.human_ui_state(&self.profile.player) {
            Some(TentativeState::from(&self.game.board, ui))
        } else {
            None
        };
    }

    pub fn time_passes_for_interaction_state(&mut self, delta: Milliseconds) -> Vec<ClientLocalEvent> {
        use self::ClientLocalEvent::*;

        let mut events : Vec<ClientLocalEvent> = Vec::new();

        match self.game.interactivity {
            InteractionState::AwaitingInput { .. } => (),
            InteractionState::WaitingVictory { ref player, ref mut elapsed } => {
                *elapsed += delta;
                if *elapsed >= VICTORY_WAIT {
                    if player.is_human(&self.profile.player) {
                        events.push(PlayerWin);
                    } else {
                        events.push(PlayerLoss);
                    }
                }
            }
            InteractionState::AnimatingMove { ref mut elapsed, ref winner, .. } => {
                *elapsed += delta;
                if *elapsed >= ANIMATION_WAIT {
                    let is = if let &Some(ref player) = winner {
                        InteractionState::WaitingVictory { player: player.clone(), elapsed: 0 }
                    } else {
                        InteractionState::awaiting_input(&self.game.board.state(), &self.game.players)
                    };
                    events.push(NewInteractionState(is));
                }
            }
        }

        events
    }

    pub fn respond_to_human_input(&self, input_state: &InputState) -> Vec<ClientLocalEvent> {
        use self::ClientLocalEvent::*;

        let mut events : Vec<ClientLocalEvent> = Vec::new();

        match self.game.interactivity {
            InteractionState::AwaitingInput { player: Player::AI }  => {
                if let Some(ref analysis) = self.game.analysis {
                    if analysis.terminal {
                        if let Some((mve, h)) = analysis.best_move {
                            if self.game.board.legal_moves().iter().any(|&m| m == mve) {
                                events.push(PlayMove(mve, Some(h)));
                            }
                        }
                    }
                }
            },
            InteractionState::AwaitingInput { player: Player::Human(_) } => { // if &human_player == &self.profile.player
                // if we have ui state
                if let Some(ui) = self.game.players.human_ui_state(&self.profile.player) {
                    let tentative = TentativeState::from(&self.game.board, ui);
                    
                    if input_state.mouse.left_released() { // if user clicked on tentative slot
                        if let Some(sl) = ui.tentative_slot {
                            if tentative.move_count > 0 {
                                events.push(PushCurrentSlot(sl));
                            }
                        }
                    } else if input_state.mouse.right_released() && !ui.current_slots.is_empty() {
                        events.push(PopCurrentSlot);
                    }
                } else {
                    println!("FOR SOME REASON WE HAVE NO UI STATE :-/ should be impossible");
                }
            },
            InteractionState::WaitingVictory { .. } => (),
            InteractionState::AnimatingMove { .. } => (),
        }

        events
    }

    pub fn update(&mut self, intersection: Option<Vec3>, input_state: &InputState, sound_event_sink: &mut Vec<SoundEvent>, delta_time: Seconds) {
        use self::ClientLocalEvent::*;

        let mut evs = Vec::new();

        let mouse_over_slot : Option<Slot> = intersection.and_then(|intersects_at| {
            Position::from(intersects_at.x - 1.0, intersects_at.z - 1.0).and_then(|p| self.board.slot_for(p) )
        });

        if let Some(ui) = self.game.players.human_ui_state(&self.profile.player)  {
            if ui.tentative_slot != mouse_over_slot {
                evs.push(UpdateTentativeSlot(mouse_over_slot));
            }
        }

        self.process_multiple(evs);

        let time_pass_events = self.time_passes_for_interaction_state((delta_time * 1000.0) as Milliseconds);
        self.process_multiple(time_pass_events);

        let human_events = self.respond_to_human_input(input_state);
        self.process_multiple(human_events);
        
        'ai_loop: loop {
            match self.ai_service.receive.try_recv() {
                Ok(analysis) => {
                    self.process(NewAnalysis(analysis));
                }
                Err(_) => {
                    break 'ai_loop;
                }
            }
        }

        sound_event_sink.append(&mut self.sound_events);
    }

    pub fn request_ai_analysis(&self) {
        if let Some(ai_profile) = self.game.players.first_ai() {
            self.ai_service.request_analysis(self.game.board.state().clone(), ai_profile);       
        }
    }

    pub fn play_move(&mut self, mve: Move) -> MatchStatus {
        let prior_state = self.game.board.state().clone();

        let player_moving = self.game.players.for_player(self.game.board.state().to_move).clone();

        self.game.board.make_move(&self.board, mve);

        let match_status = self.game.board.match_status(&self.board);
        
        let winning_player : Option<Player> = match match_status {
            MatchStatus::Won(ref player) => Some(self.game.players.0[player.0 as usize].0.clone()),
            MatchStatus::ToMove(_) => None,
        };

        self.game.interactivity = InteractionState::AnimatingMove { 
            prior_state: prior_state.clone(), 
            mve: mve, 
            player: player_moving, 
            elapsed : 0, 
            winner: winning_player
        };

        if let Some(ui) = self.game.players.mut_human_ui_state(&self.profile.player) {
            ui.clear();
        }

        self.game.tentative = None;
        self.game.analysis = None;

        if self.game.waiting_on_ai() {
            self.request_ai_analysis()
        }
        
        match_status
    }

    pub fn complete_game(&mut self, player_win: bool) { // RESET AINT GOOD ENOUGH ANYMORE
        if player_win {
            self.profile.progress.win();
        } else {
            self.profile.progress.loss();
        }
        
        let res = aphid::serialize_to_json_file::<_,_,aphid::codec::JsonCodec>(&self.profile, &self.profile_path);
        println!("serialization result -> {:?}", res);

        let game_players = self.profile.progress.players(&self.profile.player);

        self.game = game_for(&self.board, game_players);
        self.ai_service.reset();

        if self.game.waiting_on_ai() {
            self.request_ai_analysis();
        }
    }

    pub fn ui_status(&self) -> (String, String) {
        let progress = format!("Depth {} - Wins {}/{}", self.profile.progress.level, self.profile.progress.wins, wins_to_pass_for_level(self.profile.progress.level));
        let status : String = match self.game.interactivity {
            InteractionState::AwaitingInput { player: Player::AI, .. } => "Waiting on AI Opponent ...".into(),
            InteractionState::AwaitingInput { player: Player::Human(_), .. } => "Your move.".into(),
            InteractionState::WaitingVictory { ref player, .. } => {
                match player {
                    &Player::Human(_) => "Victory!".into(),
                    &Player::AI =>  "Defeat!".into(),
                }
            },
            InteractionState::AnimatingMove { .. } => "Moving ...".into(),
        };
        (progress, status)
    }

    pub fn render(&self, tesselator: &mut GeometryTesselator, opaque: &mut Vec<Vertex>, trans: &mut Vec<Vertex>, units_per_point: f64) {
    	tesselator.draw_floor_tile(opaque, &self.atlas.background, 0.0, 0.0, 0.0, 0.0);

        let next_player_color = PLAYER_COLORS[self.game.board.state().player().0 as usize];

        match &self.game.interactivity {
            &InteractionState::AnimatingMove { player: Player::Human(_), .. } => {
                 self.draw_opaques(&self.game.board.state(), tesselator, opaque, units_per_point)
            }
            &InteractionState::AnimatingMove { ref prior_state, mve, player:Player::AI, elapsed, .. } => {
                let subtracted_state = subtract(prior_state, mve);
                self.draw_opaques(&subtracted_state, tesselator, opaque, units_per_point);

                let progress = clamp((elapsed as f64) / (ANIMATION_WAIT as f64), 0.0, 1.0);
                match mve {
                    Move::PlaceBuilders { a, b } => {
                        for slot in vec![a, b] {
                            let v = Self::exact_position(&subtracted_state, slot, units_per_point);
                            tesselator.color = [1.0, 1.0, 1.0, progress as f32];
                            tesselator.draw_floor_tile_at(trans, &self.atlas.players[subtracted_state.to_move.0 as usize], v, 0.15);
                        }
                    },
                    Move::Move { from, to, build } => {
                        let from_position = Self::exact_position(&subtracted_state, from, units_per_point);
                        let to_position = Self::exact_position(&subtracted_state, to, units_per_point);
                        let exact_position = from_position.lerp(to_position, progress);
                        tesselator.color = WHITE.float_raw();
                        tesselator.draw_floor_tile_at(trans, &self.atlas.players[subtracted_state.to_move.0 as usize], exact_position, 0.15);

                        // progress for fading in building
                        tesselator.color = [1.0, 1.0, 1.0, progress as f32];
                        let building_height = subtracted_state.get_building_height(build);
                        let is_dome = building_height == 3;

                        let pos = StandardBoard::position(build);
                        

                        if is_dome {
                            let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                            v.y += (BUILDING_PIXEL_OFFSETS[3] as f64) * units_per_point;
                            tesselator.draw_floor_tile_at(trans, &self.atlas.dome, v, 0.10);
                        } else {
                            let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                            v.y += (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
                            tesselator.draw_floor_tile_at(trans, &self.atlas.buildings[building_height as usize], v, 0.10);
                        }
                    },
                }
            },
            &InteractionState::AwaitingInput { player: Player::AI, .. } => {
                self.draw_opaques(&self.game.board.state(), tesselator, opaque, units_per_point);
            },
            &InteractionState::AwaitingInput { player: Player::Human(_), ..  } => {
                if let Some(ref tentative) = self.game.tentative {
                    self.draw_opaques(&tentative.proposed_state, tesselator, opaque, units_per_point);
                    for slot in &tentative.matching_slots {
                        let pos = StandardBoard::position(*slot);
                        let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
                        tesselator.color = next_player_color.float_raw();
                        tesselator.draw_floor_tile_at(trans, &self.atlas.indicator, v, 0.1);
                    }
                } else {
                    self.draw_opaques(&self.game.board.state(), tesselator, opaque, units_per_point);
                }
            },
            &InteractionState::WaitingVictory { .. } => {
                self.draw_opaques(&self.game.board.state(), tesselator, opaque, units_per_point);
            },
        }

        

        // map(|t| t.proposed_state.clone() )

        tesselator.color = WHITE.float_raw();

         // DRAW MOUSE OVER

        if let Some(ui) = self.game.players.human_ui_state(&self.profile.player) {
            if let Some(slot) = ui.tentative_slot {
                let position = StandardBoard::position(slot);
                let v = Vec3::new(position.x as f64, 0.0, position.y as f64) + BOARD_OFFSET;
                tesselator.color = color::WHITE.float_raw();
                tesselator.draw_floor_tile_at(opaque, &self.atlas.indicator, v, 0.12);
            }
        }
    }

    pub fn exact_position(state:&State, slot:Slot, units_per_point: f64) -> Vec3 {
        let pos = StandardBoard::position(slot);
        let mut v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;
        let building_height = state.get_building_height(slot);
        v.y += (BUILDING_PIXEL_OFFSETS[building_height as usize] as f64) * units_per_point;
        v
    }

    pub fn draw_opaques(&self, state: &State, tesselator: &GeometryTesselator, vertices: &mut Vec<Vertex>, units_per_point: f64) {
        // DRAW BOARD CONTENTS
        for &slot in &self.board.slots {
            let pos = StandardBoard::position(slot);
            let v = Vec3::new(pos.x as f64, 0.0, pos.y as f64) + BOARD_OFFSET;

            let building_height = state.get_building_height(slot);
            let dome = state.domes.get(slot) == 1;

            // RENDER THE BUILDING
            for i in 0..building_height {
                let vert_offset = (BUILDING_PIXEL_OFFSETS[i as usize] as f64) * units_per_point;
                tesselator.draw_floor_tile_at(vertices, &self.atlas.buildings[i as usize], v + Vec3::new(0.0, vert_offset, 0.0), 0.10)
            }
            // RENDER THE DOME
            if dome {
                let vert_offset = (BUILDING_PIXEL_OFFSETS[3] as f64) * units_per_point;
                tesselator.draw_floor_tile_at(vertices, &self.atlas.dome, v + Vec3::new(0.0, vert_offset, 0.0), 0.10)
            }
        }

        // DRAW THE GUYS
        for (player_id, builders) in state.builders.iter().enumerate() {
            for slot in builders.iter() {
                if slot != UNPLACED_BUILDER {
                    let v = Self::exact_position(state, slot, units_per_point);
                    tesselator.draw_floor_tile_at(vertices, &self.atlas.players[player_id as usize],  v, 0.15);
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
        let grid = TextureAtlas { texture_size: 512, tile_size: 16 };

        SantoriniAtlas {
            background: grid.get(0, 0, 7, 8),
            buildings: [grid.at(7, 0), grid.at(7, 1), grid.at(7, 2)],
            dome: grid.at(7, 3),
            players: [grid.at(8, 0), grid.at(8, 1)],
            indicator: grid.at(9, 1),
        }
    }
}