pub struct Subject {
    pub code: String,
    pub name: String,
    pub credit: f64,
    pub result: Outcome,
    /// Whether this subject counts toward the average / valid credits.
    /// Lets a subject be kept in the list (e.g. a retake) without affecting
    /// the calculation.
    pub included: bool,
    /// A hypothetical grade (e.g. from a planned retake) shown alongside the
    /// actual one and factored into the potential average.
    pub potential: Option<Grade>,
}

pub struct Semester {
    pub number: usize,
    pub subjects: Vec<Subject>,
}

pub enum Outcome {
    Passed(Option<Grade>),
    Failed,
}

#[derive(Copy, Clone)]
pub enum Grade {
    A,
    B,
    C,
    D,
    E,
}

impl From<Grade> for usize {
    fn from(grade: Grade) -> Self {
        match grade {
            Grade::A => 5,
            Grade::B => 4,
            Grade::C => 3,
            Grade::D => 2,
            Grade::E => 1,
        }
    }
}
