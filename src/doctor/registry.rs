use super::{Check, CheckResult, Environment, Report};

#[derive(Default)]
pub struct Registry {
    checks: Vec<Box<dyn Check>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn register(&mut self, check: Box<dyn Check>) {
        self.checks.push(check);
    }

    pub fn survey(&self, env: &Environment) -> Report {
        let results = self
            .checks
            .iter()
            .map(|check| CheckResult {
                id: check.id().to_string(),
                section: check.section(),
                outcome: check.run(env),
            })
            .collect();

        Report {
            gave_up: None,
            results,
        }
    }
}
