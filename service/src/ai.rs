
use tavern_core::game::santorini::*;
use tavern_core::game::util::*;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

use std::thread;
use std::thread::JoinHandle;

use aphid::contains;

pub struct AIService {
    send: Sender<Request>,
    pub receive: Receiver<StateAnalysis>,
    join_handle: JoinHandle<()>,
}

#[derive(Clone)]
pub enum Request {
    Reset,
    Analysis { state: State, ai_profile: AIProfile, time_limit : Option<f64> },
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct StateAnalysis {
    pub state: State,
    pub depth: Depth,
    pub best_move: Option<(Move, HeuristicValue)>,
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

            let board = StandardBoard::new(ZobristHash::new_unseeded());

            let mut evaluator_state = NegaMaxAlphaBetaExp::new_state();

            while let Some(event) = ai_rx.recv().ok() {
                match event {
                    Reset => {
                        NegaMaxAlphaBetaExp::reset(&mut evaluator_state);
                    },
                    Analysis { state, ai_profile, time_limit } => {
                        println!("Starting analysis with ai_profile -> {:?}", ai_profile);
                        match ai_profile.heuristic {
                            HeuristicName::Simple => AIService::evaluate::<NegaMaxAlphaBetaExp, SimpleHeightHeuristic>(&mut evaluator_state, &board, &state, ai_profile.depth, time_limit, &ai_tx),
                            HeuristicName::Neighbour => AIService::evaluate::<NegaMaxAlphaBetaExp, NeighbourHeuristic>(&mut evaluator_state, &board, &state, ai_profile.depth, time_limit, &ai_tx),
                            HeuristicName::AdjustedNeighbour => AIService::evaluate::<NegaMaxAlphaBetaExp, AdjustedNeighbourHeuristic>(&mut evaluator_state, &board, &state, ai_profile.depth, time_limit, &ai_tx),
                        }
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

    pub fn winning_player(heuristic_value:HeuristicValue) -> Option<Player> {
        if heuristic_value == PLAYER_0_WIN {
            Some(Player(0))
        } else if heuristic_value == PLAYER_1_WIN {
            Some(Player(1))
        } else {
            None
        }
    }

    pub fn evaluate<E, H>(evaluator_state: &mut E::EvaluatorState, board: &StandardBoard, state:&State, max_depth:Depth, time_limit: Option<f64>, send: &Sender<StateAnalysis>) where E: Evaluator, H: Heuristic {

        E::new_search(evaluator_state);

        let score = H::evaluate(board, state);
        println!("AI :: Asked for analysis, current score {:?} with {:?} to move", score, state.to_move);
        
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
                let terminate = depth >= 2 && (depth >= max_depth || contains(time_limit, |&tl| next_timing_calc > tl));
                println!("depth is {:?} max {} terminate? {:?}", depth, max_depth, terminate);
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

    pub fn request_analysis(&self, state: State, ai_profile: AIProfile, time_limit : Option<f64>) {
        let request = Request::Analysis {
            state: state,
            ai_profile: ai_profile,
            time_limit: time_limit,
        };
        self.send.send(request).expect("can send analysis request to ai worker");
    }

    pub fn reset(&self) {
        self.send.send(Request::Reset).expect("that i can send a reset");
    }

    pub fn shutdown(self) {
        self.send.send(Request::Shutdown).expect("can send shutdown to ai worker");
        self.join_handle.join().unwrap();
    }
}