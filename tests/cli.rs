use std::process::Command;

fn grapheina() -> Command {
    Command::new(env!("CARGO_BIN_EXE_grapheina"))
}

#[test]
fn doctor_on_this_machine_prints_the_report_and_exits_zero() {
    let output = grapheina().arg("doctor").output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("grapheina doctor"));
    assert!(stdout.contains("System"));
    assert!(stdout.contains("git"));
    assert!(stdout.contains("cargo"));
}

#[test]
fn doctor_exits_two_when_a_check_is_broken() {
    let output = grapheina()
        .arg("doctor")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\u{2717} git"));
    assert!(stdout.contains("\u{2717} cargo"));
    assert!(stdout.contains("fix: install git"));
    assert!(stdout.contains("fix: install Rust (rustup)"));
}

#[test]
fn no_command_prints_usage_and_exits_sixty_four() {
    let output = grapheina().output().unwrap();

    assert_eq!(output.status.code(), Some(64));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("usage: grapheina doctor"));
}

#[test]
fn an_unknown_command_prints_usage_and_exits_sixty_four() {
    let output = grapheina().arg("dance").output().unwrap();

    assert_eq!(output.status.code(), Some(64));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("dance"));
    assert!(stderr.contains("usage: grapheina doctor"));
}
