use std::ops::{Add, AddAssign};
use game::santorini::*;

#[derive(Clone, Debug)]
pub struct EvaluatorInfo {
    pub move_count : MoveCount,
    pub branch_factors : Vec<f64>,
    pub time : f64,
}

impl EvaluatorInfo {
    pub fn new() -> EvaluatorInfo {
        EvaluatorInfo {
            move_count: 0,
            branch_factors: Vec::new(),
            time: 0.0,
        }
    }

    pub fn average_branch_factor(&self) -> BranchFactor {
        average(&self.branch_factors)
    }

    pub fn from_moves_depth(move_count: MoveCount, depth: u8) -> EvaluatorInfo {
        EvaluatorInfo {
            move_count : move_count,
            branch_factors : vec![branch_factor(move_count, depth)],
            time : 0.0_f64,
        }
    }

    pub fn moves_per_second(&self) -> f64 {
        if self.time > 0.0 {
            self.move_count as f64 / self.time
        } else {
            0.0
        }
    }
}

impl Add for EvaluatorInfo {
    type Output = EvaluatorInfo;

    fn add(self, other: EvaluatorInfo) -> EvaluatorInfo {
        let mut self_branch = self.branch_factors;
        self_branch.extend_from_slice(&other.branch_factors);
        EvaluatorInfo {
            move_count : self.move_count + other.move_count,
            branch_factors : self_branch,
            time : self.time + other.time,
        }
    }
}

impl AddAssign for EvaluatorInfo {
    fn add_assign(&mut self, other: EvaluatorInfo) {
        self.move_count += other.move_count;
        self.branch_factors.extend_from_slice(&other.branch_factors);
        self.time += other.time;
    }
}

pub fn branch_factor(move_count: MoveCount, depth: u8) -> f64 {
    (move_count as f64).powf(1.0 / (depth as f64))
}

pub fn average(arr: &[f64]) -> f64 {
    let n = arr.len() as f64;
    arr.iter().fold(0.0, |acc, val| acc + val) / n
}