


fn run_service() {
    let mut service = ServiceState::new(RunnerDescription::RunnerA);
    let msg_a = service.runner.run("_mymsg1_".into());
    println!("msg a -> {}", msg_a);
    service.reconfigure(RunnerDescription::RunnerB);
    let msg_b = service.runner.run("_mymsg2_".into());
    println!("msg b -> {}", msg_b);
    service.reconfigure(RunnerDescription::RunnerB);
}

struct ServiceState {
    pub description: RunnerDescription,
    pub runner: Box<Runner>,
}

impl ServiceState {
    pub fn new(runner_description: RunnerDescription) -> ServiceState {
        ServiceState {
            description: runner_description,
            runner: runner_for_description(runner_description),
        }
    }

    pub fn reconfigure(&mut self, runner_description: RunnerDescription) {
        if runner_description != self.description {
            self.runner = runner_for_description(runner_description);
            self.description = runner_description;
        } 
    }
}

fn runner_for_description(runner_description: RunnerDescription) -> Box<Runner> {
    match runner_description {
        RunnerDescription::RunnerA => Box::new(RunnerA {}),
        RunnerDescription::RunnerB => Box::new(RunnerB::new(24)),
    }
}



pub trait Scorer {
    fn score() -> u64;
}

struct BigScorer {}
impl Scorer for BigScorer {
    fn score() -> u64 { 24 }
}

struct SmallScorer {}
impl Scorer for SmallScorer {
    fn score() -> u64 { 12 }
}

pub trait Runner {
    // fn run<S>(&mut self, msg:String) -> String where S : Scorer;
    fn run(&mut self, msg:String) -> String ; // where Self:Sized
}

struct RunnerA {}

impl Runner for RunnerA {
    // fn run<S>(&mut self, msg:String) -> String where S : Scorer {
    //     format!("AAA {} -> {}", msg, S::score())
    // }
    fn run(&mut self, msg:String) -> String { // where  Self:Sized 
        format!("AAA {}", msg)
    }
}

struct RunnerB {
    pub count: u64
}
impl RunnerB {
    fn new(use_count: u64) -> Self {
        RunnerB {
            count: use_count,
        }
    }
}
impl Runner for RunnerB {
    // fn run<S>(&mut self, msg:String) -> String where S : Scorer {
    //     format!("BBBB {} -> {}", msg, S::score())
    // }
    fn run(&mut self, msg:String) -> String { //  where Self:Sized 
        format!("BBBB {}", msg)
    }
}

#[derive(Eq, Debug, Copy, Clone, PartialEq)]
enum RunnerDescription {
    RunnerA,
    RunnerB
}