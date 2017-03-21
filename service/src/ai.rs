
use tavern_core::game::santorini::*;
use tavern_core::game::util::*;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

use std::thread;
use std::thread::JoinHandle;

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
    pub best_move: Option<(Move, HeuristicValue)>,
    pub terminal: bool, 
    pub rollback : bool, // rollback means we discovered we will lose guaranteed ... so we resort to the prior depth to discovering that ... so we still make a reasonable move
}

impl AIService {
    pub fn new<E>() -> AIService where E: Evaluator {
        use self::Request::*;

        let (main_tx, ai_rx) = channel::<Request>();
        let (ai_tx, main_rx) = channel::<StateAnalysis>();

        let join_handle = thread::spawn(move || {
            println!("ai server started");

            let board = StandardBoard::new(ZobristHash::new_unseeded());

            let mut evaluator_state = E::new_state();

            while let Some(event) = ai_rx.recv().ok() {
                match event {
                    Analysis(state) => {
                        // NegaMax
                        // NegaMaxAlphaBeta
                        AIService::evaluate::<E, NeighbourHeuristic>(&mut evaluator_state, &board, &state, &ai_tx);
                    },
                    Shutdown => {
                        println!("Ai shutdown requested");
                        break;  
                    }
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

    pub fn evaluate<E, H>(evaluator_state: &mut E::EvaluatorState, board: &StandardBoard, state:&State, send: &Sender<StateAnalysis>) where E: Evaluator, H: Heuristic {
        println!("AI :: Asked for analysis");
        // println!("{}", board.print(&state));
        let score = SimpleHeightHeuristic::evaluate(board, state) * Self::player_multiplier(state.to_move);
        println!("AI :: current score it as -> {:?} with {:?} to move", score, state.to_move);

        let max_depth = if state.builders_to_place() {
            5
        }  else {
            6
        };

        for depth in 1..(max_depth+1) {
            let (best_move, info) = E::evaluate_moves::<H>(evaluator_state, board, state, depth);  

            let best_move_score = best_move.map(|(_, score)| score);
            let winning_player = best_move_score.and_then(|score| AIService::winning_player(score));

            println!("AI :: depth {:?} info {:?} best_move -> {:?}", depth, info, best_move);

            if let Some(player) = winning_player {
                println!("AI :: at depth {:?} we've established winning player will be {:?}", depth, player);

                if player != state.to_move && depth > 0 {
                    println!("AI :: we've lost, and we've got a rollback state, rolling back, performing sample playout");
                    // playout::<E, H>(board, state, depth);
                    send.send(StateAnalysis {
                        state: state.clone(),
                        depth: depth,
                        best_move: best_move,
                        terminal: true, 
                        rollback: true,
                    }).unwrap();
                } else {
                    send.send(StateAnalysis {
                        state: state.clone(),
                        depth: depth,
                        best_move: best_move,
                        terminal: true, 
                        rollback: false,
                    }).unwrap();
                }
                break;
            } else {
                let next_timing_calc = info.time * (info.average_branch_factor() as f64);
                println!("we're at depth {} time was {:.3} next timing calc is {:.3}", depth, info.time, next_timing_calc);
                let terminate = depth == max_depth || next_timing_calc > 30.0;
                send.send(StateAnalysis {
                    state: state.clone(),
                    depth: depth,
                    best_move: best_move,
                    terminal: terminate, 
                    rollback: false,
                }).unwrap();
                if terminate {
                    break;
                }
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