use super::{Outcome, Section, Status};

pub struct CheckResult {
    pub id: String,
    pub section: Section,
    pub outcome: Outcome,
}

pub struct Report {
    pub gave_up: Option<String>,
    pub results: Vec<CheckResult>,
}

impl Report {
    pub fn exit_code(&self) -> u8 {
        if self.gave_up.is_some() {
            return 6;
        }

        let mut code = 0;
        for result in &self.results {
            match result.outcome.status {
                Status::Broken => return 2,
                Status::Degraded => code = 4,
                Status::Fine => {}
            }
        }
        code
    }
}
