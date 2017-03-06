
use game::santorini::*;

impl StandardBoard {
    pub fn perft(&self, state: &State, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut n = 0;

        let mut moves = Vec::new();
        self.next_moves(state, &mut moves);
        for mve in &moves {
            if self.ascension_winning_move(state, *mve) {
                n += 1;
            } else {
                let new_state = self.apply(*mve, state);
                n += self.perft(&new_state, depth - 1);
            }
        }
        
        n
    }
}