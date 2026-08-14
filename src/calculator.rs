pub struct Subject {
    pub name: String,
    pub credit: f64,
    pub semester: usize,
    pub result: SubjectResult,
}

pub enum SubjectResult {
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

struct CalcModel {
    grade: Grade,
    weight: f64,
}

impl From<Subject> for Option<CalcModel> {
    fn from(subject: Subject) -> Self {
        match subject.result {
            SubjectResult::Passed(Some(grade)) => Some(CalcModel {
                grade,
                weight: subject.credit,
            }),
            _ => None,
        }
    }
}

fn ects_average(grades: &[CalcModel]) -> f64 {
    let (weighted_credit_sum, credit_sum) = grades.iter().fold(
        (0.0, 0.0),
        |(sum, credits), &CalcModel { grade, weight }| {
            let grade_value = usize::from(grade) as f64;
            (sum + grade_value * weight, credits + weight)
        },
    );
    if credit_sum == 0.0 {
        return 0.0;
    }
    weighted_credit_sum / credit_sum
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::calculator::Grade::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_ects_average() {
        let grades = &[CalcModel {
            grade: A,
            weight: 10.0,
        }];
        assert_relative_eq!(ects_average(grades), 5.0);

        let grades: &[CalcModel; 0] = &[];
        assert_relative_eq!(ects_average(grades), 0.0);
    }
}
