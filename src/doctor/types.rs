#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    System,
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Fine,
    Degraded,
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadius {
    Read,
    InsideProject,
    OutsideProject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub action: String,
    pub touches: BlastRadius,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub status: Status,
    pub detail: String,
    pub fix: Option<Fix>,
}
