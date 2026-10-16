use super::{BlastRadius, Fix, Report, Section, Status};

impl Report {
    pub fn render(&self) -> String {
        let mut out = String::from("grapheina doctor\n");

        if let Some(reason) = &self.gave_up {
            out.push_str(&format!("\n  gave up: {reason}\n"));
            return out;
        }

        for section in [Section::System, Section::Code] {
            out.push('\n');
            out.push_str(match section {
                Section::System => "  System\n",
                Section::Code => "  Code\n",
            });

            let results: Vec<_> = self
                .results
                .iter()
                .filter(|result| result.section == section)
                .collect();

            if results.is_empty() {
                out.push_str("    (no checks registered)\n");
                continue;
            }

            let width = results
                .iter()
                .map(|result| result.id.len())
                .max()
                .unwrap_or(0);
            for result in results {
                let symbol = match result.outcome.status {
                    Status::Fine => '\u{2713}',
                    Status::Degraded => '\u{26a0}',
                    Status::Broken => '\u{2717}',
                };
                out.push_str(&format!(
                    "    {symbol} {:width$}  {}\n",
                    result.id, result.outcome.detail
                ));
                if let Some(fix) = &result.outcome.fix {
                    out.push_str(&format!(
                        "        fix: {} \u{2014} {}\n",
                        fix.action,
                        boundary_note(fix)
                    ));
                }
            }
        }

        out
    }
}

fn boundary_note(fix: &Fix) -> &'static str {
    match fix.touches {
        BlastRadius::Read => "read-only, never asks",
        BlastRadius::InsideProject => "inside the project, runs without asking",
        BlastRadius::OutsideProject => "outside the project, always asks first",
    }
}
