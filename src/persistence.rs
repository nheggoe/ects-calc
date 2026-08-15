use crate::model::{Grade, Outcome, Semester, Subject};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

type Document = BTreeMap<String, Vec<TomlSubject>>;

#[derive(Serialize, Deserialize)]
struct TomlSubject {
    #[serde(default)]
    code: String,
    name: String,
    credit: f64,
    grade: String,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    included: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    potential: Option<String>,
}

fn default_true() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid toml: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("failed to serialize toml: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid grade: {0:?}")]
    InvalidGrade(String),
    #[error("invalid semester key: {0:?}")]
    InvalidSemesterKey(String),
}

pub fn default_path() -> Result<PathBuf, Error> {
    let dirs = directories::ProjectDirs::from("", "", "ects-calc").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine data directory",
        )
    })?;
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("subjects.toml"))
}

pub fn load(path: &Path) -> Result<Vec<Semester>, Error> {
    let raw = std::fs::read_to_string(path)?;
    let doc: Document = toml::from_str(&raw)?;
    parse_document(doc)
}

pub fn save(path: &Path, semesters: &[Semester]) -> Result<(), Error> {
    let doc = to_document(semesters);
    let raw = toml::to_string_pretty(&doc)?;
    std::fs::write(path, raw)?;
    Ok(())
}

fn parse_document(doc: Document) -> Result<Vec<Semester>, Error> {
    doc.into_iter()
        .map(|(key, subjects)| {
            let number = parse_semester_key(&key)?;
            let subjects = subjects
                .into_iter()
                .map(|s| {
                    Ok(Subject {
                        code: s.code,
                        name: s.name,
                        credit: s.credit,
                        result: parse_grade(&s.grade)?,
                        included: s.included,
                        potential: s.potential.as_deref().map(parse_grade_letter).transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(Semester { number, subjects })
        })
        .collect()
}

fn to_document(semesters: &[Semester]) -> Document {
    semesters
        .iter()
        .map(|semester| {
            let key = format!("semester{}", semester.number);
            let subjects = semester
                .subjects
                .iter()
                .map(|subject| TomlSubject {
                    code: subject.code.clone(),
                    name: subject.name.clone(),
                    credit: subject.credit,
                    grade: format_grade(&subject.result),
                    included: subject.included,
                    potential: subject.potential.map(|g| grade_letter(g).to_string()),
                })
                .collect();
            (key, subjects)
        })
        .collect()
}

fn parse_semester_key(key: &str) -> Result<usize, Error> {
    key.strip_prefix("semester")
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| Error::InvalidSemesterKey(key.to_string()))
}

pub fn parse_grade(grade: &str) -> Result<Outcome, Error> {
    match grade.to_ascii_lowercase().as_str() {
        "a" => Ok(Outcome::Passed(Some(Grade::A))),
        "b" => Ok(Outcome::Passed(Some(Grade::B))),
        "c" => Ok(Outcome::Passed(Some(Grade::C))),
        "d" => Ok(Outcome::Passed(Some(Grade::D))),
        "e" => Ok(Outcome::Passed(Some(Grade::E))),
        "pass" | "passed" => Ok(Outcome::Passed(None)),
        "f" | "fail" | "failed" => Ok(Outcome::Failed),
        _ => Err(Error::InvalidGrade(grade.to_string())),
    }
}

pub fn format_grade(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Passed(Some(grade)) => grade_letter(*grade),
        Outcome::Passed(None) => "Pass",
        Outcome::Failed => "F",
    }
    .to_string()
}

pub fn parse_grade_letter(s: &str) -> Result<Grade, Error> {
    match s.to_ascii_lowercase().as_str() {
        "a" => Ok(Grade::A),
        "b" => Ok(Grade::B),
        "c" => Ok(Grade::C),
        "d" => Ok(Grade::D),
        "e" => Ok(Grade::E),
        _ => Err(Error::InvalidGrade(s.to_string())),
    }
}

pub fn grade_letter(grade: Grade) -> &'static str {
    match grade {
        Grade::A => "A",
        Grade::B => "B",
        Grade::C => "C",
        Grade::D => "D",
        Grade::E => "E",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::Grade::*;

    #[test]
    fn round_trip_document() {
        let semesters = vec![Semester {
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
                    included: false,
                    potential: Some(C),
                },
            ],
        }];

        let doc = to_document(&semesters);
        let parsed = parse_document(doc).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].number, 1);
        assert_eq!(parsed[0].subjects.len(), 3);
        assert!(parsed[0].subjects[0].included);
        assert!(!parsed[0].subjects[2].included);
        assert!(parsed[0].subjects[0].potential.is_none());
        assert!(matches!(parsed[0].subjects[2].potential, Some(C)));
    }

    #[test]
    fn missing_potential_field_defaults_to_none() {
        let doc: Document = toml::from_str(
            r#"
            [[semester1]]
            name = "Math"
            credit = 10.0
            grade = "A"
            "#,
        )
        .unwrap();
        let semesters = parse_document(doc).unwrap();
        assert!(semesters[0].subjects[0].potential.is_none());
    }

    #[test]
    fn rejects_invalid_potential_grade() {
        assert!(parse_grade_letter("X").is_err());
        assert!(parse_grade_letter("Pass").is_err());
    }

    #[test]
    fn missing_included_field_defaults_to_true() {
        let doc: Document = toml::from_str(
            r#"
            [[semester1]]
            name = "Math"
            credit = 10.0
            grade = "A"
            "#,
        )
        .unwrap();
        let semesters = parse_document(doc).unwrap();
        assert!(semesters[0].subjects[0].included);
    }

    #[test]
    fn rejects_invalid_grade() {
        assert!(parse_grade("X").is_err());
    }

    #[test]
    fn parses_grade_case_insensitively() {
        assert!(matches!(
            parse_grade("a").unwrap(),
            Outcome::Passed(Some(A))
        ));
        assert!(matches!(
            parse_grade("pass").unwrap(),
            Outcome::Passed(None)
        ));
        assert!(matches!(
            parse_grade("Passed").unwrap(),
            Outcome::Passed(None)
        ));
        assert!(matches!(parse_grade("fail").unwrap(), Outcome::Failed));
        assert!(matches!(parse_grade("FAILED").unwrap(), Outcome::Failed));
    }

    #[test]
    fn rejects_invalid_semester_key() {
        assert!(parse_semester_key("sem1").is_err());
    }

    #[test]
    fn save_and_load_file() {
        let semesters = vec![Semester {
            number: 2,
            subjects: vec![Subject {
                code: "PH1".into(),
                name: "Physics".into(),
                credit: 7.5,
                result: Outcome::Passed(Some(B)),
                included: true,
                potential: Some(A),
            }],
        }];

        let path = std::env::temp_dir().join("ects_calc_persistence_test.toml");
        save(&path, &semesters).unwrap();
        let loaded = load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].number, 2);
        assert_eq!(loaded[0].subjects[0].name, "Physics");
        assert!(matches!(loaded[0].subjects[0].potential, Some(A)));
    }
}
