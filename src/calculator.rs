use crate::model::{Grade, Outcome, Semester, Subject};

struct WeightedGrade {
    grade: Grade,
    weight: f64,
}

impl From<&Subject> for Option<WeightedGrade> {
    fn from(subject: &Subject) -> Self {
        if !subject.included {
            return None;
        }
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

/// Same as `overall_average`, but a subject's `potential` grade (if set)
/// stands in for its actual one — including turning a currently-failed or
/// not-yet-passed subject into a graded one, as a "what if" projection.
pub fn potential_average(semesters: &[Semester]) -> f64 {
    let grades: Vec<WeightedGrade> = semesters
        .iter()
        .flat_map(|semester| &semester.subjects)
        .filter(|subject| subject.included)
        .filter_map(|subject| {
            let grade = subject.potential.or(match subject.result {
                Outcome::Passed(Some(grade)) => Some(grade),
                _ => None,
            })?;
            Some(WeightedGrade {
                grade,
                weight: subject.credit,
            })
        })
        .collect();
    ects_average(&grades)
}

pub fn valid_credits(semesters: &[Semester]) -> f64 {
    semesters
        .iter()
        .flat_map(|semester| &semester.subjects)
        .filter(|subject| subject.included && !matches!(subject.result, Outcome::Failed))
        .map(|subject| subject.credit)
        .fold(0.0, |acc, credit| acc + credit)
}

/// Same as `valid_credits`, but a failed subject with a `potential` grade
/// set counts too — projecting the credits earned if that "what if" grade
/// came true.
pub fn potential_valid_credits(semesters: &[Semester]) -> f64 {
    semesters
        .iter()
        .flat_map(|semester| &semester.subjects)
        .filter(|subject| subject.included)
        .filter(|subject| !matches!(subject.result, Outcome::Failed) || subject.potential.is_some())
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
                    code: "MA1".into(),
                    name: "Math".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(A)),
                    included: true,
                    potential: None,
                },
                Subject {
                    code: "ET1".into(),
                    name: "Ethics".into(),
                    credit: 5.0,
                    result: Outcome::Passed(None),
                    included: true,
                    potential: None,
                },
                Subject {
                    code: "DB1".into(),
                    name: "Databases".into(),
                    credit: 10.0,
                    result: Outcome::Failed,
                    included: true,
                    potential: None,
                },
            ],
        }];
        assert_relative_eq!(valid_credits(semesters), 15.0);
    }

    #[test]
    fn excluded_subjects_are_ignored_by_average_and_valid_credits() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![
                Subject {
                    code: "MA1".into(),
                    name: "Math".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(A)),
                    included: true,
                    potential: None,
                },
                Subject {
                    code: "RETAKE".into(),
                    name: "Old attempt".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(E)),
                    included: false,
                    potential: None,
                },
            ],
        }];
        assert_relative_eq!(overall_average(semesters), 5.0);
        assert_relative_eq!(valid_credits(semesters), 10.0);
    }

    #[test]
    fn valid_credits_of_empty_is_positive_zero() {
        let credits = valid_credits(&[]);
        assert_relative_eq!(credits, 0.0);
        assert!(!credits.is_sign_negative());
    }

    #[test]
    fn potential_average_uses_potential_grade_when_set() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![
                Subject {
                    code: "MA1".into(),
                    name: "Math".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(C)),
                    included: true,
                    potential: Some(A),
                },
                Subject {
                    code: "ET1".into(),
                    name: "Ethics".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(C)),
                    included: true,
                    potential: None,
                },
            ],
        }];
        assert_relative_eq!(overall_average(semesters), 3.0);
        assert_relative_eq!(potential_average(semesters), 4.0);
    }

    #[test]
    fn potential_average_can_turn_a_failed_subject_into_a_pass() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![Subject {
                code: "DB1".into(),
                name: "Databases".into(),
                credit: 10.0,
                result: Outcome::Failed,
                included: true,
                potential: Some(D),
            }],
        }];
        assert_relative_eq!(overall_average(semesters), 0.0);
        assert_relative_eq!(potential_average(semesters), 2.0);
    }

    #[test]
    fn potential_average_matches_actual_when_nothing_is_set() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![Subject {
                code: "MA1".into(),
                name: "Math".into(),
                credit: 10.0,
                result: Outcome::Passed(Some(B)),
                included: true,
                potential: None,
            }],
        }];
        assert_relative_eq!(potential_average(semesters), overall_average(semesters));
    }

    #[test]
    fn potential_valid_credits_counts_a_failed_subject_with_a_potential_grade() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![
                Subject {
                    code: "MA1".into(),
                    name: "Math".into(),
                    credit: 10.0,
                    result: Outcome::Passed(Some(A)),
                    included: true,
                    potential: None,
                },
                Subject {
                    code: "DB1".into(),
                    name: "Databases".into(),
                    credit: 5.0,
                    result: Outcome::Failed,
                    included: true,
                    potential: Some(D),
                },
                Subject {
                    code: "ET1".into(),
                    name: "Ethics".into(),
                    credit: 7.0,
                    result: Outcome::Failed,
                    included: true,
                    potential: None,
                },
            ],
        }];
        assert_relative_eq!(valid_credits(semesters), 10.0);
        assert_relative_eq!(potential_valid_credits(semesters), 15.0);
    }

    #[test]
    fn potential_valid_credits_matches_actual_when_nothing_is_set() {
        let semesters = &[Semester {
            number: 1,
            subjects: vec![Subject {
                code: "MA1".into(),
                name: "Math".into(),
                credit: 10.0,
                result: Outcome::Passed(Some(B)),
                included: true,
                potential: None,
            }],
        }];
        assert_relative_eq!(potential_valid_credits(semesters), valid_credits(semesters));
    }
}
