use std::ops::{Add, AddAssign};
use game::santorini::*;
use std::fmt;

#[derive(Clone)]
pub struct EvaluatorInfo {
    pub move_count : MoveCount,
    pub pv_count : MoveCount,
    pub branch_factors : Vec<f64>,
    pub time : f64,
    pub tt_valid : u64,
    pub tt_suggest: u64,
    pub tt_miss : u64,
}

impl EvaluatorInfo {
    pub fn new() -> EvaluatorInfo {
        EvaluatorInfo {
            move_count: 0,
            pv_count: 0,
            branch_factors: Vec::new(),
            time: 0.0,
            tt_valid : 0,
            tt_suggest: 0,
            tt_miss : 0,
        }
    }

    pub fn average_branch_factor(&self) -> BranchFactor {
        average(&self.branch_factors)
    }

    pub fn from_moves_depth(move_count: MoveCount, depth: Depth) -> EvaluatorInfo {
        EvaluatorInfo {
            move_count : move_count,
            pv_count: 0,
            branch_factors : vec![branch_factor(move_count, depth)],
            time : 0.0_f64,
            tt_valid : 0,
            tt_suggest: 0,
            tt_miss : 0,
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

impl fmt::Debug for EvaluatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let average_branch_factor = average(&self.branch_factors);
        let moves_per_second = self.move_count as f64 / self.time / 1000000.0;
        let pv_percentage = self.pv_count as f64 / (self.move_count as f64);
        write!(f, "EvaluatorInfo {{ moves: {} ({:.2}M/second) pv nodes: {} ({:.2}%) average branch factor: {:.1} time: {:0.2}s transpotition (valid {} sugg {} miss {})}}", 
            self.move_count, moves_per_second, self.pv_count, pv_percentage, average_branch_factor, self.time,
            self.tt_valid,
            self.tt_suggest,
            self.tt_miss,
        )
    }
}

impl Add for EvaluatorInfo {
    type Output = EvaluatorInfo;

    fn add(self, other: EvaluatorInfo) -> EvaluatorInfo {
        let mut self_branch = self.branch_factors;
        self_branch.extend_from_slice(&other.branch_factors);
        EvaluatorInfo {
            move_count : self.move_count + other.move_count,
            pv_count: self.pv_count + other.pv_count,
            branch_factors : self_branch,
            time : self.time + other.time,
            tt_valid : self.tt_valid + other.tt_valid,
            tt_suggest: self.tt_suggest + other.tt_suggest,
            tt_miss : self.tt_miss + other.tt_miss,
        }
    }
}

impl AddAssign for EvaluatorInfo {
    fn add_assign(&mut self, other: EvaluatorInfo) {
        self.move_count += other.move_count;
        self.pv_count += other.pv_count;
        self.branch_factors.extend_from_slice(&other.branch_factors);
        self.time += other.time;
        self.tt_valid += other.tt_valid;
        self.tt_suggest += other.tt_suggest;
        self.tt_miss += other.tt_miss;
    }
}

pub fn branch_factor(move_count: MoveCount, depth: Depth) -> f64 {
    (move_count as f64).powf(1.0 / (depth as f64))
}

pub fn average(arr: &[f64]) -> f64 {
    let n = arr.len() as f64;
    arr.iter().fold(0.0, |acc, val| acc + val) / n
}