

use jam::camera::Camera;
use jam::render::glium::renderer::Renderer;

use jam::*;
use jam::render::*;


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
        zoom: 1.0,
        points_per_unit: 16.0,
        n: 0, // frame counter
        renderer: renderer,
        state: Game::Santorini(SantoriniGame::new()),
    };
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
}

pub struct SantoriniGame {
	pub board: santorini::StandardBoard,
	// rest is per game, more transient
	pub state: santorini::State,
	pub cpu_players : HashSet<Slot>,
	pub move_builder : MoveBuilder,
}

impl SantoriniGame {
	pub fn new() -> SantoriniGame {
		SantoriniGame {
			board: santorini::StandardBoard::new(),
			state: santorini::State::initial(),
			cpu_players: HashSet::default(),
			move_builder : MoveBuilder { positions: vec![] },
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
    	println!("update");

    }

    fn render(&mut self) -> Vec<Pass<String>> {
    	println!("render");
    	vec![]
    }
}