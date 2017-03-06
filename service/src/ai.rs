
use tavern_core::game::santorini::*;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::SendError;

use std::thread;
use std::thread::JoinHandle;

pub struct AIService {
	send: Sender<Request>,
	receive: Receiver<StateAnalysis>,
	join_handle: JoinHandle<()>,
}

pub enum Request {
	Analysis(State),
	Shutdown,
}

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
                    			AIService::evaluate(&board, &state);

                    			
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

	pub fn evaluate(board: &StandardBoard, state:&State) {
		println!("AI Worker has been asked for analysis");
		println!("{}", board.print(&state));
		let score = SimpleHeightHeuristic::evaluate(board, state);
		println!("we score it as -> {:?}", score);



	}

	pub fn request_analysis(&self, state: &State) {
		self.send.send(Request::Analysis(state.clone())).expect("can send analysis request to ai worker");
	}

	pub fn shutdown(self) {
		self.send.send(Request::Shutdown).expect("can send shutdown to ai worker");
		self.join_handle.join().unwrap();
	}
}