
use game::santorini::*;

pub fn mild_a_advantage(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    state.buildings = state.buildings.set(Slot(5), 1);
    state.to_move = to_move;
    state
}

pub fn distant_state(board:&StandardBoard) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(23), b: Slot(24) }] {
        state = board.apply(mve, &state);
    }
    state
}


pub fn a_in_1(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 0
    state.buildings = state.buildings.set(Slot(1), 2);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.to_move = to_move;
    state
}

pub fn a_in_2(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 0
    state.buildings = state.buildings.set(Slot(1), 1);
    state.buildings = state.buildings.set(Slot(2), 2);
    state.buildings = state.buildings.set(Slot(3), 3);
    state.to_move = to_move;
    state
}

pub fn b_in_1(board: &StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 1
    state.buildings = state.buildings.set(Slot(23), 2);
    state.buildings = state.buildings.set(Slot(22), 3);

    state.to_move = to_move;
    state
}

pub fn b_in_2(board: &StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 1
    state.buildings = state.buildings.set(Slot(23), 1);
    state.buildings = state.buildings.set(Slot(22), 2);
    state.buildings = state.buildings.set(Slot(21), 3);
    state.to_move = to_move;
    state
}

pub fn a_blockable(board: &StandardBoard, to_move: Player) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(3), b: Slot(4) }] {
        state = board.apply(mve, &state);
    }
    state.buildings = state.buildings.set(Slot(1), 2);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.buildings = state.buildings.set(Slot(3), 1); // we put the B player up a bit so it must move down to sacrifice
    state.to_move = to_move;
    state
}

// pub fn b_blockable_win(board: &StandardBoard, to_move: Player) -> State {
    
// }

// pub fn to_move_win(board:&StandardBoard, to_move: Player) -> State {

// }

// trapping


pub fn evaluate<E, H>(board:&StandardBoard, state:&State, max_depth: u8)  -> Vec<HeuristicValue> where E: Evaluation, H:Heuristic {
    (1..(max_depth+1)).flat_map(|depth| {
        E::evaluate::<H>(board, state, depth).iter().map(|&(_, sc)| sc).take(1).collect::<Vec<_>>()
    }).collect()
}

pub struct TestCase {
    pub name: String,
    pub state: State,
    pub scores: Vec<HeuristicValue>,
}

pub fn case(name:&str, state:State, scores:Vec<HeuristicValue>) -> TestCase {
    TestCase {
        name: name.into(),
        state: state,
        scores: scores
    }
}

pub fn test_cases(board:&StandardBoard) -> Vec<TestCase> {
    vec![
        case("mild_a", mild_a_advantage(board, Player(0)), vec![1,1,2,1]),
        case("mild_a", mild_a_advantage(board, Player(1)), vec![0,1,0,1]),

        case("a_in_1", a_in_1(board, Player(0)), vec![PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_1", a_in_1(board, Player(1)), vec![2, PLAYER_0_WIN]),

        case("a_in_2", a_in_2(board, Player(0)), vec![2,2,PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_2", a_in_2(board, Player(1)), vec![1,2,1, PLAYER_0_WIN] ),

        case("b_in_1", b_in_1(board, Player(0)), vec![-2, PLAYER_1_WIN] ),
        case("b_in_1", b_in_1(board, Player(1)), vec![PLAYER_1_WIN, PLAYER_1_WIN] ),

        case("b_in_2", b_in_2(board, Player(0)), vec![-1,-2,-1, PLAYER_1_WIN] ),
        case("b_in_2", b_in_2(board, Player(1)), vec![-2,-2,PLAYER_1_WIN, PLAYER_1_WIN] ),

        case("a_blockable", a_blockable(board, Player(0)), vec![PLAYER_0_WIN] ),
        case("a_blockable", a_blockable(board, Player(1)), vec![1, 1] ),
    ]
}

mod minimax {
    use super::*;

    #[test]
    fn all() {
        let board = StandardBoard::new();
        for case in &test_cases(&board) {
            println!("Testing {} to move {:?}", case.name, case.state.to_move);
            assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &case.state, case.scores.len() as u8), case.scores); 
        }
    }

    // #[test]
    fn test_playout() {
        // let board = StandardBoard::new();
        // let state = &a_unavoidable_win_in_2(&board, Player(0));
        // playout::<MiniMax, SimpleHeightHeuristic>(&board, &state, 3);
    }
}


mod negamax {
    use super::*;

    #[test]
    fn mild_a_moves() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<NegaMax, SimpleHeightHeuristic>(&board, &mild_a_advantage(&board, Player(0)), 4), vec![1,1,2,1]); 
    }

    #[test]
    fn mild_b_moves() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<NegaMax, SimpleHeightHeuristic>(&board, &mild_a_advantage(&board, Player(1)), 4), vec![0,1,0,1]); 
    }
}
    

