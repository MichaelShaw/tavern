
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


pub fn b_in_2(board: &StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 1
    state.buildings = state.buildings.set(Slot(23), 1);
    state.buildings = state.buildings.set(Slot(22), 2);
    state.buildings = state.buildings.set(Slot(21), 3);
    state.to_move = to_move;
    state
}

// pub fn a_blockable_win(board: &StandardBoard, to_move: Player) -> State {

// }

// pub fn b_blockable_win(board: &StandardBoard, to_move: Player) -> State {
    
// }

// pub fn to_move_win(board:&StandardBoard, to_move: Player) -> State {

// }


pub fn evaluate<E, H>(board:&StandardBoard, state:&State, max_depth: u8)  -> Vec<HeuristicValue> where E: Evaluation, H:Heuristic {
    (1..(max_depth+1)).flat_map(|depth| {
        E::evaluate::<H>(board, state, depth).iter().map(|&(_, sc)| sc).take(1).collect::<Vec<_>>()
    }).collect()
}

mod minimax {
    use super::*;

    #[test]
    fn test_playout() {
        // let board = StandardBoard::new();
        // let state = &a_unavoidable_win_in_2(&board, Player(0));
        // playout::<MiniMax, SimpleHeightHeuristic>(&board, &state, 3);
    }

    #[test]
    fn mild_a_moves() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &mild_a_advantage(&board, Player(0)), 4), vec![1,1,2,1]); 
    }

    #[test]
    fn mild_b_moves() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &mild_a_advantage(&board, Player(1)), 4), vec![0,1,0,1]); 
    }

    #[test]
    fn a_win_in_1() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &a_in_1(&board, Player(0)), 2), vec![PLAYER_0_WIN, PLAYER_0_WIN]); 
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &a_in_1(&board, Player(1)), 2), vec![2, PLAYER_0_WIN]); 
    }

    #[test]
    fn a_win_in_2() {
        let board = StandardBoard::new();
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &a_in_2(&board, Player(0)), 4), vec![2,2,PLAYER_0_WIN, PLAYER_0_WIN]); 
        assert_eq!(evaluate::<MiniMax, SimpleHeightHeuristic>(&board, &a_in_2(&board, Player(1)), 4), vec![1,2,1, PLAYER_0_WIN]); 
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
    

