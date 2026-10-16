use grapheina::doctor::{
    BlastRadius, Check, Environment, Fix, Outcome, Probe, Registry, Report, Section, Status,
    builtin,
};

struct Stub {
    id: &'static str,
    section: Section,
    status: Status,
    detail: &'static str,
    fix: Option<grapheina::doctor::Fix>,
}

impl Check for Stub {
    fn id(&self) -> &str {
        self.id
    }

    fn section(&self) -> Section {
        self.section
    }

    fn run(&self, _env: &Environment) -> Outcome {
        Outcome {
            status: self.status,
            detail: self.detail.to_string(),
            fix: self.fix.clone(),
        }
    }
}

fn stub(id: &'static str, status: Status, detail: &'static str) -> Box<dyn Check> {
    Box::new(Stub {
        id,
        section: Section::System,
        status,
        detail,
        fix: None,
    })
}

fn stub_with_fix(id: &'static str, status: Status, detail: &'static str) -> Box<dyn Check> {
    Box::new(Stub {
        id,
        section: Section::System,
        status,
        detail,
        fix: Some(Fix {
            action: "install Rust (rustup)".to_string(),
            touches: BlastRadius::OutsideProject,
        }),
    })
}

fn no_programs() -> Environment {
    Environment::from_probe(|_| Probe::Missing)
}

#[test]
fn the_builtin_registry_checks_git_and_cargo_on_a_prepared_machine() {
    let env = Environment::from_probe(|program| match program {
        "git" => Probe::Found {
            version: "2.45.0".to_string(),
        },
        "cargo" => Probe::Found {
            version: "1.99.1".to_string(),
        },
        _ => Probe::Missing,
    });

    let report = builtin().survey(&env);

    assert_eq!(report.exit_code(), 0);
    let ids: Vec<(&str, Section)> = report
        .results
        .iter()
        .map(|result| (result.id.as_str(), result.section))
        .collect();
    assert_eq!(
        ids,
        vec![("git", Section::System), ("cargo", Section::System)]
    );
    assert_eq!(report.results[0].outcome.detail, "2.45.0");
    assert_eq!(report.results[1].outcome.detail, "1.99.1");
}

#[test]
fn a_missing_program_is_broken_with_a_system_fix() {
    let env = Environment::from_probe(|program| match program {
        "git" => Probe::Found {
            version: "2.45.0".to_string(),
        },
        _ => Probe::Missing,
    });

    let report = builtin().survey(&env);

    assert_eq!(report.exit_code(), 2);
    let cargo = report
        .results
        .iter()
        .find(|result| result.id == "cargo")
        .unwrap();
    assert_eq!(cargo.outcome.status, Status::Broken);
    assert_eq!(cargo.outcome.detail, "not installed");
    let fix = cargo.outcome.fix.as_ref().unwrap();
    assert_eq!(fix.action, "install Rust (rustup)");
    assert_eq!(fix.touches, BlastRadius::OutsideProject);
}

#[test]
fn a_report_renders_statuses_sections_and_the_fix_boundary() {
    let mut registry = Registry::new();
    registry.register(stub("git", Status::Fine, "2.45.0"));
    registry.register(stub_with_fix("cargo", Status::Broken, "not installed"));

    let report = registry.survey(&no_programs());

    let expected = "grapheina doctor\n\n  System\n    \u{2713} git    2.45.0\n    \u{2717} cargo  not installed\n        fix: install Rust (rustup) \u{2014} outside the project, always asks first\n\n  Code\n    (no checks registered)\n";
    assert_eq!(report.render(), expected);
}

#[test]
fn a_given_up_report_renders_the_reason() {
    let report = Report {
        gave_up: Some("could not read the machine".to_string()),
        results: Vec::new(),
    };

    assert_eq!(
        report.render(),
        "grapheina doctor\n\n  gave up: could not read the machine\n"
    );
}

#[test]
fn a_given_up_doctor_makes_exit_code_six() {
    let report = Report {
        gave_up: Some("could not finish checking".to_string()),
        results: Vec::new(),
    };

    assert_eq!(report.exit_code(), 6);
}

#[test]
fn an_accepted_degradation_makes_exit_code_four() {
    let mut registry = Registry::new();
    registry.register(stub("sandbox", Status::Degraded, "no container, accepted"));

    let report = registry.survey(&no_programs());

    assert_eq!(report.exit_code(), 4);
}

#[test]
fn an_unaccepted_break_outweighs_an_accepted_degradation() {
    let mut registry = Registry::new();
    registry.register(stub("sandbox", Status::Degraded, "no container, accepted"));
    registry.register(stub("cargo", Status::Broken, "not installed"));

    let report = registry.survey(&no_programs());

    assert_eq!(report.exit_code(), 2);
}

#[test]
fn a_broken_check_makes_exit_code_two() {
    let mut registry = Registry::new();
    registry.register(stub("cargo", Status::Broken, "not installed"));

    let report = registry.survey(&no_programs());

    assert_eq!(report.exit_code(), 2);
}

#[test]
fn a_fine_check_reports_fine_and_exit_code_zero() {
    let mut registry = Registry::new();
    registry.register(stub("git", Status::Fine, "2.45.0"));

    let report = registry.survey(&no_programs());

    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].id, "git");
    assert_eq!(report.results[0].outcome.status, Status::Fine);
    assert_eq!(report.results[0].outcome.detail, "2.45.0");
}
