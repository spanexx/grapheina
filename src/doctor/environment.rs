#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Found { version: String },
    Missing,
}

pub struct Environment {
    probe: Box<dyn Fn(&str) -> Probe>,
}

impl Environment {
    pub fn real() -> Self {
        Self::from_probe(|program| {
            let output = std::process::Command::new(program)
                .arg("--version")
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    Probe::Found {
                        version: extract_version(&text),
                    }
                }
                _ => Probe::Missing,
            }
        })
    }

    pub fn from_probe(probe: impl Fn(&str) -> Probe + 'static) -> Self {
        Self {
            probe: Box::new(probe),
        }
    }

    pub fn probe(&self, program: &str) -> Probe {
        (self.probe)(program)
    }
}

pub fn extract_version(output: &str) -> String {
    output
        .split_whitespace()
        .find(|token| token.starts_with(|c: char| c.is_ascii_digit()))
        .map(|token| token.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_version;

    #[test]
    fn extracts_the_version_from_git_style_output() {
        assert_eq!(extract_version("git version 2.45.0"), "2.45.0");
    }

    #[test]
    fn extracts_the_version_from_cargo_style_output() {
        assert_eq!(
            extract_version("cargo 1.99.1 (5f94df478 2026-08-27)"),
            "1.99.1"
        );
    }

    #[test]
    fn reports_unknown_when_no_version_is_present() {
        assert_eq!(extract_version("something went wrong"), "unknown");
    }
}
