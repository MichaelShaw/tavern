

use jam::*;
use jam::render::*;
use jam::render::gfx::{OpenGLRenderer, construct_opengl_renderer};
use jam::ui::*;


use std::f64::consts::PI;

use time;

use cgmath::{Rad, Zero, vec2};

use aphid;
use aphid::{HashMap, HashSet, Seconds};

use howl::{Listener, SoundEvent};
use howl::worker::SoundWorker;
use howl::engine::SoundEngineUpdate::*;

use santorini;

//use std::env;
//use std::fs;

use std::path::PathBuf;

use std::io;

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
    NoHomeDir,
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
//        if cfg!(debug_assertions) { //
            Ok((TavernPaths {
                resources: "./resources".into(),
                openal: "./native/openal.dylib".into(),
                profile: PathBuf::from("./tavern.profile.txt"), // ~/Library/Application Support/tavern
            }))
//        } else {
//            // mac in a .app
//            let mut resources_path = env::current_exe().unwrap();
//            resources_path.pop();
//            resources_path.pop();
//            resources_path.push("Resources");
//            // let mut f = File::create(resources_path.with_file_name("my_paths.txt")).unwrap();
//
//            let r_path = resources_path.to_str().unwrap().into();
//
//            let mut alpth = resources_path.clone();
//            alpth.push("openal.dylib");
//
//            let al_path = alpth.to_str().unwrap().into();
//
//            // SAVE PROFILE IN APP SUPPORT
//            let mut home_dir = try!(env::home_dir().ok_or(TavernError::NoHomeDir));
//            home_dir.push("Library");
//            home_dir.push("Application Support");
//            home_dir.push("Tavern");
//
//            if !home_dir.exists() {
//                fs::create_dir(&home_dir)?;
//            }
//            home_dir.push("tavern.profile.txt");
//
//            Ok((TavernPaths {
//                resources: r_path,
//                openal: al_path,
//                profile: home_dir,
//            }))
//        }
    } else  {
        Ok((TavernPaths {
            resources: "./resources".into(),
            openal: "./native/OpenAL64.dll".into(),
            profile: PathBuf::from("./tavern.profile.txt"), // current directory
        }))
    }
}

use rand::XorShiftRng;

pub fn run_app() -> TavernResult<()> {
    let paths = try!(get_paths());

    println!("paths -> {:?}", paths);

    let sound_path = format!("{}/sound", paths.resources);
    let vertex_shader_path = format!("{}/shader/fat.vert", paths.resources);
    let fragment_shader_path = format!("{}/shader/fat.frag", paths.resources);
    let texture_path = format!("{}/textures", paths.resources);
    let fonts_path = format!("{}/fonts", paths.resources);

    let rng = XorShiftRng::new_unseeded();

    let sound_worker = SoundWorker::create(paths.openal, sound_path, "ogg".into(), rng, 1_000_000, 5.0);
    sound_worker.send(Preload(vec![("place_tile".into(), 1.0), ("select".into(), 1.0)])).unwrap();

    let shader_pair = ShaderPair::for_paths(&vertex_shader_path, &fragment_shader_path);
    let texture_dir = TextureDirectory::for_path(&texture_path, hashset!["png".into()]);
    let font_dir = FontDirectory::for_path(&fonts_path);

    let file_resources = FileResources {
        resources: PathBuf::from(paths.resources),
        shader_pair,
        texture_directory: texture_dir,
        font_directory: font_dir,
    };


    let renderer = construct_opengl_renderer(file_resources, (800, 600), true, "tavern".into()).expect("a renderer");

    let dimensions = Dimensions {
        pixels: (800,600),
        points: (800, 600),
    };

    let mut app = App {
        name: "mixalot".into(),
        camera: Camera {
            at: Vec3::new(0.0, 0.0, 0.0),
            pitch: Rad(PI / 4.0_f64),
            viewport: dimensions,
            points_per_unit: 16.0 * 1.0,
        },
        zoom: 4.0,
        points_per_unit: 16.0,
        n: 0, // frame counter
        renderer,
        sound_worker,
        client: santorini::SantoriniClient::new(paths.profile),
        ui: WidgetRunner::new(SantoriniUI {}, SantoriniUIState::empty(), dimensions),
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
    renderer: OpenGLRenderer,
    sound_worker: SoundWorker,
    client: santorini::SantoriniClient,
    ui: WidgetRunner<SantoriniUI>,
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

    fn ui_tesselator(&self) -> GeometryTesselator {
        GeometryTesselator::new(Vec3::new(1.0, 1.0, 1.0))
    }

    fn run(&mut self) {
        let start_time = time::precise_time_ns();
        let mut last_time = start_time;
        
        'main: loop {
            let (dimensions, input_state) = self.renderer.begin_frame(rgb(116, 181, 231));

            let time = time::precise_time_ns();
            let delta_time = ((time - last_time) as f64) / 1_000_000_000.0;
            let since_start = ((time - start_time) as f64) / 1_000_000_000.0;

            self.update(&input_state, dimensions, since_start, delta_time);  

            self.render(dimensions).expect("rendering to work");

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
        self.client.update(ground_intersection, &input_state, &mut sound_events, delta_time);
        
        self.camera.at = Vec3::new(3.5, 0.0, 3.5);
        self.camera.points_per_unit = self.points_per_unit * self.zoom;
        self.camera.viewport = dimensions;

        use howl::engine::SoundEngineUpdate;
        use howl::engine::SoundRender;

        let mut external_events = Vec::new();

        let (progress, status) = self.client.ui_status();

        let ui_state = SantoriniUIState {
            progress,
            status,
        };

        if ui_state != self.ui.state {
            external_events.push(ui_state);
        }


        self.ui.run(input_state.clone(), external_events, dimensions);

        // "song".into() => song()
        let engine_update = SoundEngineUpdate::Render(SoundRender { master_gain: 1.0, sounds:sound_events, persistent_sounds: HashMap::default(), listener: Listener::default() });
        self.sound_worker.send(engine_update).expect("the sound worker to be alive");
    }

    fn render(&mut self, _dimensions:Dimensions) -> JamResult<()> {
        let mut tesselator = self.tesselator();

        let mut opaque_vertices = Vec::new();
        let mut trans_vertices = Vec::new();

        let upp = self.units_per_point();

        self.client.render(&mut tesselator, &mut opaque_vertices, &mut trans_vertices, upp);

        self.renderer.draw_vertices(&opaque_vertices, Uniforms {
            transform : down_size_m4(self.camera.view_projection().into()),
            color: color::WHITE,
        }, Blend::None)?;

        self.renderer.draw_vertices(&trans_vertices, Uniforms {
            transform : down_size_m4(self.camera.view_projection().into()),
            color: color::WHITE,
        }, Blend::Alpha)?;

        self.renderer.clear_depth();

        self.renderer.draw_view(&self.ui.view())?;

        self.renderer.finish_frame()?;
       
        Ok(())
    }
}

pub fn song() -> SoundEvent {
    SoundEvent {
        name: "hollow_wanderer".into(),
        position: [0.0, 0.0, 0.0],
        gain: 1.0,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    }
}

#[derive(Clone,Eq, PartialEq, Debug)]
pub struct SantoriniUIState {
    pub progress: String,
    pub status: String,
}

impl SantoriniUIState {
    pub fn empty() -> SantoriniUIState {
        SantoriniUIState {
            progress: "".into(),
            status: "".into(),
        }
    }
}

pub struct SantoriniUI {

}

impl Widget for SantoriniUI {
    type State = SantoriniUIState;
    type Event = SantoriniUIState;

    #[allow(unused_variables)]
    fn update(&self, state:&SantoriniUIState, ev:&SantoriniUIState) -> SantoriniUIState {
        ev.clone()
    }

    fn view(&self, state:&SantoriniUIState, dimensions:Dimensions) -> View<SantoriniUIState> {
        let mut view = empty_view(RectI::new(vec2(20, 20), vec2(300, 140)));

        let black_text = rgb(0, 0, 0);

//        println!("generating view for state -> {:?} dimensions -> {:?}", state, dimensions);

        let padded_screen = dimensions.points_rect().padded(40);

        let top = padded_screen.top(60);

        let bottom = padded_screen.bottom(60);


        view.sub_views.push(label_view(top, state.progress.clone(), black_text, None, None));
        view.sub_views.push(label_view(bottom, state.status.clone(), black_text, None, None));

        view
    }
}
