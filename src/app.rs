

use jam::camera::Camera;
use jam::render::glium::renderer::Renderer;

use jam::*;
use jam::render::*;
use jam::render::Command::*;


use tavern_core::Slot;
// use tavern_core::game::util::Player;
use tavern_core::game::santorini;
use std::f64::consts::PI;

use jam::color::rgb;

use time;

use cgmath::Rad;

pub fn run_app() {
	let shader_pair = ShaderPair::for_paths("resources/shader/fat.vert", "resources/shader/fat.frag");
    let texture_dir = TextureDirectory::for_path("resources/textures", hashset!["png".into()]);
    let font_dir = FontDirectory::for_path("resources/fonts");

    let renderer = Renderer::new(shader_pair, texture_dir, font_dir, (800, 600)).expect("a renderer");

    let mut app = App {
        name: "mixalot".into(),
        camera: Camera {
            at: Vec3::new(0.0, 0.0, 0.0),
            pitch: Rad(PI / 4.0_f64),
            viewport: Dimensions { 
                pixels: (800,600),
                scale: 1.0,
            },
            points_per_unit: 16.0 * 1.0,
        },
        zoom: 4.0,
        points_per_unit: 16.0,
        n: 0, // frame counter
        renderer: renderer,
        state: Game::Santorini(SantoriniGame::new()),
        atlas: SantoriniAtlas::build(),
    };

    println!("atlas -> {:?}", app.atlas);
    app.run();
}

struct App {
    name : String,
    camera : Camera,
    zoom : f64,
    points_per_unit : f64,
    n : u64,
    renderer:Renderer<String>,
    state: Game,
    atlas: SantoriniAtlas,
}

pub struct SantoriniGame {
	pub board: santorini::StandardBoard,
	// rest is per game, more transient
	pub state: santorini::State,
	pub cpu_players : HashSet<Slot>,
	pub move_builder : MoveBuilder,
	pub mouse_over_slot : Option<Slot>,
}

impl SantoriniGame {
	pub fn new() -> SantoriniGame {
		SantoriniGame {
			board: santorini::StandardBoard::new(),
			state: santorini::State::initial(),
			cpu_players: HashSet::default(),
			move_builder : MoveBuilder { positions: vec![] },
			mouse_over_slot: None,
		}
	}
}

pub enum Game {
	Santorini(SantoriniGame)
}

pub struct MoveBuilder {
	pub positions : Vec<Slot>
}


impl App {
	fn units_per_point(&self) -> f64 {
        1.0 / self.points_per_unit
    }

    fn tesselator(&self) -> GeometryTesselator {
        let upp = self.units_per_point();
        let tesselator_scale = Vec3::new(upp, upp, upp);
        GeometryTesselator::new(tesselator_scale)
    }

    fn run(&mut self) {
        let mut last_time = time::precise_time_ns();
        'main: loop {
            let (dimensions, input_state) = self.renderer.begin();

            let time = time::precise_time_ns();
            let delta_time = ((time - last_time) as f64) / 1_000_000.0;

            self.update(&input_state, dimensions, delta_time);  

            let render_passes = self.render();

            self.renderer.render(render_passes, rgb(132, 193, 255));

            last_time = time;
            if input_state.close {
                break;
            }
        }
    }

    fn update(&mut self, input_state:&InputState, dimensions:Dimensions, delta_time: Seconds) {

    	self.camera.at = Vec3::new(3.5, 0.0, 3.5);
    	self.camera.points_per_unit = self.points_per_unit * self.zoom;
        self.camera.viewport = dimensions;
    }

    fn render(&mut self) -> Vec<Pass<String>> {
    	let mut opaque_commands : Vec<Command<String>> = Vec::new();
        let mut translucent_commands : Vec<Command<String>> = Vec::new();
        let mut ui_commands : Vec<Command<String>> = Vec::new();

		let mut opaque = self.tesselator();
		let mut trans = self.tesselator();

        
		let atlas = &self.atlas;

        match &self.state {
        	&Game::Santorini(SantoriniGame { ref board, ref state, ref move_builder, .. }) => {
        		opaque.draw_floor_tile(&atlas.background, 0, 0.0, 0.0, 0.0, 0.0, false);

        	},
        }


        opaque_commands.push(DrawNew {
            key: None, 
            vertices: opaque.tesselator.vertices, 
            uniforms: Uniforms {
                transform : down_size_m4(self.camera.view_projection().into()),
                color: color::WHITE,
            }
        });

        translucent_commands.push(DrawNew {
            key: None, 
            vertices: trans.tesselator.vertices, 
            uniforms: Uniforms {
                transform : down_size_m4(self.camera.view_projection().into()),
                color: color::WHITE,
            }
        });

    	vec![Pass {
            blend: Blend::None,
            commands: opaque_commands,
            clear_depth: false,
        }, Pass {
            blend: Blend::Alpha,
            commands: translucent_commands,
            clear_depth: false,
        }, Pass {
            blend: Blend::Alpha,
            commands: ui_commands,
            clear_depth: true,
        }]
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