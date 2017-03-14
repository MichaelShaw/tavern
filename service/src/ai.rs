
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
    pub rollback : bool, // rollback means we discovered we will lose guaranteed ... so we resort to the prior depth to discovering that ... so we still make a reasonable move
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
                                AIService::evaluate::<NegaMaxAlphaBeta, SimpleHeightHeuristic>(&board, &state, &ai_tx);
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

    pub fn player_multiplier(player:Player) -> HeuristicValue {
        match player {
            Player(0) => 1,
            Player(1) => -1,
            _ => -128,
        }
    }


    pub fn winning_player(heuristic_value:HeuristicValue) -> Option<Player> {
        if heuristic_value == PLAYER_0_WIN {
            Some(Player(0))
        } else if heuristic_value == PLAYER_1_WIN {
            Some(Player(1))
        } else {
            None
        }
    }

    pub fn evaluate<E, H>(board: &StandardBoard, state:&State, send: &Sender<StateAnalysis>) where E: Evaluation, H: Heuristic {
        println!("AI :: Asked for analysis");
        // println!("{}", board.print(&state));
        let score = SimpleHeightHeuristic::evaluate(board, state) * Self::player_multiplier(state.to_move);
        println!("AI :: current score it as -> {:?} with {:?} to move", score, state.to_move);

        let max_depth = if state.builders_to_place() {
            4
        }  else {
            5
        };

        let mut best_moves : Vec<(Move, HeuristicValue)> = Vec::new();

        for depth in 1..(max_depth+1) {
            let start = time::precise_time_ns();
            let (moves, move_count) = E::evaluate::<H>(board, state, depth);  

            let best_move_score = moves.get(0).map(|&(_, score)| score);
            let winning_player = best_move_score.and_then(|score| AIService::winning_player(score));
           
            let duration = time::precise_time_ns() - start;
            let as_seconds = (duration as f64) / 1_000_000_000f64;

            let average_branch_factor = branch_factor(move_count, depth);
            println!("AI :: depth {:?} evaluated in {:.3}s score -> {:?} total moves evaluationed -> {:?} branch_factor -> {:?}", depth, as_seconds, best_move_score, move_count, average_branch_factor);

            if let Some(player) = winning_player {
                println!("AI :: at depth {:?} we've established winning player will be {:?}", depth, player);

                if player != state.to_move && depth > 0 {
                    println!("AI :: we've lost, and we've got a rollback state, rolling back, performing sample playout");
                    // playout::<E, H>(board, state, depth);
                    send.send(StateAnalysis {
                        state: state.clone(),
                        depth: depth,
                        moves: best_moves,
                        terminal: true, 
                        rollback: true,
                    }).unwrap();
                } else {
                    send.send(StateAnalysis {
                        state: state.clone(),
                        depth: depth,
                        moves: moves,
                        terminal: true, 
                        rollback: false,
                    }).unwrap();
                }
                break;
            } else {
                best_moves = moves.clone();
                send.send(StateAnalysis {
                    state: state.clone(),
                    depth: depth,
                    moves: moves,
                    terminal: depth == max_depth, 
                    rollback: false,
                }).unwrap();
            }
        }

        println!("AI :: Evaluation over");
    }

    pub fn request_analysis(&self, state: &State) {
        self.send.send(Request::Analysis(state.clone())).expect("can send analysis request to ai worker");
    }

    pub fn shutdown(self) {
        self.send.send(Request::Shutdown).expect("can send shutdown to ai worker");
        self.join_handle.join().unwrap();
    }
}