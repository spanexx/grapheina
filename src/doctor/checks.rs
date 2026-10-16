use super::{BlastRadius, Check, Environment, Fix, Outcome, Probe, Registry, Section, Status};

pub struct CommandCheck {
    id: &'static str,
    program: &'static str,
    fix_action: &'static str,
}

impl CommandCheck {
    pub const fn new(id: &'static str, program: &'static str, fix_action: &'static str) -> Self {
        Self {
            id,
            program,
            fix_action,
        }
    }
}

impl Check for CommandCheck {
    fn id(&self) -> &str {
        self.id
    }

    fn section(&self) -> Section {
        Section::System
    }

    fn run(&self, env: &Environment) -> Outcome {
        match env.probe(self.program) {
            Probe::Found { version } => Outcome {
                status: Status::Fine,
                detail: version,
                fix: None,
            },
            Probe::Missing => Outcome {
                status: Status::Broken,
                detail: "not installed".to_string(),
                fix: Some(Fix {
                    action: self.fix_action.to_string(),
                    touches: BlastRadius::OutsideProject,
                }),
            },
        }
    }
}

pub fn builtin() -> Registry {
    let mut registry = Registry::new();
    registry.register(Box::new(CommandCheck::new("git", "git", "install git")));
    registry.register(Box::new(CommandCheck::new(
        "cargo",
        "cargo",
        "install Rust (rustup)",
    )));
    registry
}
