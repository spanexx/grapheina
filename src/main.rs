use std::io::Write;
use std::process::ExitCode;

use grapheina::doctor;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("doctor") => doctor(),
        Some(other) => usage(&format!("unknown command: {other}")),
        None => usage("no command given"),
    }
}

fn doctor() -> ExitCode {
    let report = doctor::builtin().survey(&doctor::Environment::real());

    print!("{}", report.render());
    let _ = std::io::stdout().flush();

    ExitCode::from(report.exit_code())
}

fn usage(reason: &str) -> ExitCode {
    eprintln!("grapheina: {reason}");
    eprintln!("usage: grapheina doctor");
    ExitCode::from(64)
}
