pub struct Subject {
    pub name: String,
    pub credit: f64,
    pub result: Outcome,
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
