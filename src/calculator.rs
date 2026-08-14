use crate::model::{Grade, Outcome, Semester, Subject};

struct WeightedGrade {
    grade: Grade,
    weight: f64,
}

impl From<&Subject> for Option<WeightedGrade> {
    fn from(subject: &Subject) -> Self {
        match subject.result {
            Outcome::Passed(Some(grade)) => Some(WeightedGrade {
                grade,
                weight: subject.credit,
            }),
            _ => None,
        }
    }
}

pub fn overall_average(semesters: &[Semester]) -> f64 {
    let grades: Vec<WeightedGrade> = semesters
        .iter()
        .flat_map(|semester| &semester.subjects)
        .filter_map(Option::<WeightedGrade>::from)
        .collect();
    ects_average(&grades)
}

pub fn valid_credits(semesters: &[Semester]) -> f64 {
    semesters
        .iter()
        .flat_map(|semester| &semester.subjects)
        .filter(|subject| !matches!(subject.result, Outcome::Failed))
        .map(|subject| subject.credit)
        .fold(0.0, |acc, credit| acc + credit)
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

    #[test]
    fn valid_credits_excludes_failed_subjects() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![
                Subject {
                    name: "Math".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(A)),
                },
                Subject {
                    name: "Ethics".into(),
                    credit: 5.0,
                    result: Outcome::Passed(None),
                },
                Subject {
                    name: "Databases".into(),
                    credit: 10.0,
                    result: Outcome::Failed,
                },
            ],
        }];
        assert_relative_eq!(valid_credits(semesters), 15.0);
    }

    #[test]
    fn valid_credits_of_empty_is_positive_zero() {
        let credits = valid_credits(&[]);
        assert_relative_eq!(credits, 0.0);
        assert!(!credits.is_sign_negative());
    }
}
