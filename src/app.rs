

use jam::camera::Camera;
use jam::render::glium::renderer::Renderer;

use jam::*;
use jam::render::*;
use jam::render::Command::*;


use std::f64::consts::PI;

use jam::color::rgb;

use time;

use cgmath::{Rad, Zero};

use howl::{Listener, SoundEvent, Vec3f, HashMap};
use howl::worker::SoundWorker;
use howl::engine::SoundEngineUpdate::*;

use santorini;

use std::env;

use std::path::PathBuf;

use std::io;

use aphid;

#[derive(Eq, Debug, Clone, PartialEq)]
pub struct TavernPaths {
    pub resources: String,
    pub openal : String,
    pub profile : PathBuf,
}


#[derive(Debug)]
pub enum TavernError {
    Aphid(aphid::AphidError),
    IO(io::Error),
}

impl From<aphid::AphidError> for TavernError {
    fn from(err: aphid::AphidError) -> Self {
        TavernError::Aphid(err)
    }
}

impl From<io::Error> for TavernError {
    fn from(err: io::Error) -> Self {
        TavernError::IO(err)
    }
}

pub type TavernResult<T> = Result<T, TavernError>;

pub fn get_paths() -> TavernResult<TavernPaths> {
    if cfg!(all(target_os = "macos")) { // -- mac release
        if cfg!(debug_assertions) { //
            Ok((TavernPaths {
                resources: "./resources".into(),
                openal: "./native/openal.dylib".into(),
                profile: PathBuf::from("./tavern.profile.txt"), // ~/Library/Application Support/tavern
            }))
        } else {
            // mac in a .app
            let mut resources_path = env::current_exe().unwrap();
            resources_path.pop();
            resources_path.pop();
            resources_path.push("Resources");
            // let mut f = File::create(resources_path.with_file_name("my_paths.txt")).unwrap();  

            let r_path = resources_path.to_str().unwrap().into();

            let mut alpth = resources_path.clone();
            alpth.push("openal.dylib");

            let al_path = alpth.to_str().unwrap().into();
            
            Ok((TavernPaths {
                resources: r_path,
                openal: al_path,
                profile: PathBuf::from("./tavern.profile.txt"), // ~/Library/Application Support/tavern
            }))
        }
    } else  {
        Ok((TavernPaths {
            resources: "./resources".into(),
            openal: "./native/OpenAL64.dll".into(),
            profile: PathBuf::from("./tavern.profile.txt"), // current directory
        }))
    }
}


pub fn run_app() -> TavernResult<()> {
    let paths = try!(get_paths());

    println!("paths -> {:?}", paths);

    let sound_path = format!("{}/sound", paths.resources);
    let vertex_shader_path = format!("{}/shader/fat.vert", paths.resources);
    let fragment_shader_path = format!("{}/shader/fat.frag", paths.resources);
    let texture_path = format!("{}/textures", paths.resources);
    let fonts_path = format!("{}/fonts", paths.resources);


    let sound_worker = SoundWorker::create(paths.openal, sound_path, "ogg".into(), 1_000_000, 5.0);
    sound_worker.send(Preload(vec![("place_tile".into(), 1.0), ("select".into(), 1.0)])).unwrap();

    let shader_pair = ShaderPair::for_paths(&vertex_shader_path, &fragment_shader_path);
    let texture_dir = TextureDirectory::for_path(&texture_path, hashset!["png".into()]);
    let font_dir = FontDirectory::for_path(&fonts_path);

    let renderer = Renderer::new(shader_pair, texture_dir, font_dir, (800, 600), "tavern".into()).expect("a renderer");

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
        state: Game::Santorini(santorini::SantoriniGame::new(paths.profile)),
        sound_worker: sound_worker, 
    };

    app.run();

    app.sound_worker.shutdown_and_wait();

    Ok(())
}

struct App {
    name : String,
    camera : Camera,
    zoom : f64,
    points_per_unit : f64,
    n : u64,
    renderer:Renderer<String>,
    state: Game,
    sound_worker: SoundWorker,
}

pub enum Game {
    Santorini(santorini::SantoriniGame)
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
        let start_time = time::precise_time_ns();
        let mut last_time = start_time;
        
        'main: loop {
            let (dimensions, input_state) = self.renderer.begin();

            let time = time::precise_time_ns();
            let delta_time = ((time - last_time) as f64) / 1_000_000_000.0;
            let since_start = ((time - start_time) as f64) / 1_000_000_000.0;

            self.update(&input_state, dimensions, since_start, delta_time);  

            let render_passes = self.render();

            // rgb(132, 193, 255)
            // 
            self.renderer.render(render_passes, rgb(116, 181, 231)).unwrap();

            last_time = time;
            if input_state.close {
                break;
            }
        }
    }

    #[allow(unused_variables)]
    fn update(&mut self, input_state:&InputState, dimensions:Dimensions, time: Seconds, delta_time: Seconds) {
        let mut sound_events : Vec<SoundEvent> = Vec::new();
        
        let ground_plane = Plane::from_origin_normal(Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
        let (mx, my) = input_state.mouse.at;

        let ground_intersection = self.camera.world_line_segment_for_mouse_position(mx, my).and_then(|ls| ls.intersects(ground_plane));
        match &mut self.state {
            &mut Game::Santorini(ref mut game) => {
                game.update(ground_intersection, &input_state, &mut sound_events, delta_time);
            },
        }

        self.camera.at = Vec3::new(3.5, 0.0, 3.5);
        self.camera.points_per_unit = self.points_per_unit * self.zoom;
        self.camera.viewport = dimensions;

        // "song".into() => song()
        self.sound_worker.send(Render { master_gain: 1.0, sounds:sound_events, persistent_sounds: HashMap::default(), listener: Listener::default() }).unwrap();
    }

    fn render(&mut self) -> Vec<Pass<String>> {
        let mut opaque_commands : Vec<Command<String>> = Vec::new();
        let mut translucent_commands : Vec<Command<String>> = Vec::new();
        let mut ui_commands : Vec<Command<String>> = Vec::new();

        let mut opaque = self.tesselator();
        let mut trans = self.tesselator();

        let upp = self.units_per_point();

        match &mut self.state {
            &mut Game::Santorini(ref mut game) => {
                game.render(&mut opaque, &mut trans, upp);
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

pub fn song() -> SoundEvent {
    SoundEvent {
        name: "hollow_wanderer".into(),
        position: Vec3f::zero(),
        gain: 1.0,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    }
}
