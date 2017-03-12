
use tavern_core::game::santorini::*;
use tavern_core::game::util::*;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

use std::thread;
use std::thread::JoinHandle;

use time;

pub struct AIService {
	send: Sender<Request>,
	pub receive: Receiver<StateAnalysis>,
	join_handle: JoinHandle<()>,
}

pub enum Request {
	Analysis(State),
	Shutdown,
}

#[derive(Debug, Clone)]
pub struct StateAnalysis {
	pub state: State,
	pub depth: u8,
	pub moves: Vec<(Move, HeuristicValue)>,
	pub terminal: bool, 
}

impl AIService {
	pub fn new() -> AIService {
		use self::Request::*;

		let (main_tx, ai_rx) = channel::<Request>();
		let (ai_tx, main_rx) = channel::<StateAnalysis>();

		let join_handle = thread::spawn(move || {
			println!("ai server started");

			let board = StandardBoard::new();

			loop {
                match ai_rx.recv() {
                    Ok(event) => {
                    	match event {
                    		Analysis(state) => {
                    			AIService::evaluate(&board, &state, &ai_tx);
                    		},
							Shutdown => {
								println!("Ai shutdown requested");
								break;	
							}
                    	}
                    }
                    Err(recv_error) => {
                        println!("Sound worker received error when reading from channel {:?}", recv_error);
                        break;
                    },
                }
            }
		});

		AIService {
			send: main_tx,
			receive: main_rx,
			join_handle: join_handle,
		}
	}

	pub fn player_multiplier(player:Player) -> i8 {
		match player {
			Player(0) => 1,
			Player(1) => -1,
			_ => -128,
		}
	}

	pub fn evaluate(board: &StandardBoard, state:&State, send: &Sender<StateAnalysis>) {
		println!("AI Worker has been asked for analysis");
		println!("{}", board.print(&state));
		let score = SimpleHeightHeuristic::evaluate(board, state) * Self::player_multiplier(state.to_move);
		println!("we score it as -> {:?} to move {:?}", score, state.to_move);

		let max_depth = if state.builders_to_place() {
			3
		}  else {
			5
		};
		for depth in 1..(max_depth+1) {
			let start = time::precise_time_ns();
	        let mut moves = Negamax::evaluate::<SimpleHeightHeuristic>(board, state, depth); 	
			moves.sort_by_key(|&(_, hv)| -hv);

			let best_move_score = moves.get(0).map(|&(_, score)| score);

			send.send(StateAnalysis {
				state: state.clone(),
				depth: depth,
				moves: moves,
				terminal: depth == max_depth, 
			}).unwrap();	
			let duration = time::precise_time_ns() - start;
			let as_seconds = (duration as f64) / 1_000_000_000f64;
			println!("depth {:?} evaluated in {:.3}s score -> {:?}", depth, as_seconds, best_move_score);
		}

		println!("Evaluation has concluded");
	}

	pub fn request_analysis(&self, state: &State) {
		self.send.send(Request::Analysis(state.clone())).expect("can send analysis request to ai worker");
	}

	pub fn shutdown(self) {
		self.send.send(Request::Shutdown).expect("can send shutdown to ai worker");
		self.join_handle.join().unwrap();
	}
}