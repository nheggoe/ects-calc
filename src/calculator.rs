use crate::model::{Grade, Outcome, Subject};

struct WeightedGrade {
    grade: Grade,
    weight: f64,
}

impl From<Subject> for Option<WeightedGrade> {
    fn from(subject: Subject) -> Self {
        match subject.result {
            Outcome::Passed(Some(grade)) => Some(WeightedGrade {
                grade,
                weight: subject.credit,
            }),
            _ => None,
        }
    }
}

fn ects_average(grades: &[WeightedGrade]) -> f64 {
    if grades.is_empty() {
        return 0.0;
    }
    let (weighted_credit_sum, credit_sum) = grades.iter().fold(
        (0.0, 0.0),
        |(sum, credits), &WeightedGrade { grade, weight }| {
            let grade_value = usize::from(grade) as f64;
            (sum + grade_value * weight, credits + weight)
        },
    );
    weighted_credit_sum / credit_sum
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::Grade::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_ects_average() {
        let grades = &[WeightedGrade {
            grade: A,
            weight: 10.0,
        }];
        assert_relative_eq!(ects_average(grades), 5.0);

        let grades: &[WeightedGrade; 0] = &[];
        assert_relative_eq!(ects_average(grades), 0.0);
    }
}
