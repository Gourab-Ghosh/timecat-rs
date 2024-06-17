use super::*;

#[derive(Default)]
pub struct TimecatBuilder {
    engine: Option<Engine>,
}

impl TimecatBuilder {
    pub fn build(self) -> Timecat {
        Timecat {
            engine: self.engine.unwrap_or_default(),
        }
    }
}

pub struct Timecat {
    engine: Engine,
}

impl Timecat {
    pub fn run(self, args: &[&str]) {
        Parser::parse_args_and_run_main_loop(&args);
    }
}
