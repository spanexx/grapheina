use super::{Environment, Outcome, Section};

pub trait Check {
    fn id(&self) -> &str;
    fn section(&self) -> Section;
    fn run(&self, env: &Environment) -> Outcome;
}
