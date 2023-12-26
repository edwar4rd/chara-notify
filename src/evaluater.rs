use async_trait::async_trait;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

pub type TagName = String;
pub type TagConfidence = f32;

#[async_trait]
pub trait Evaluater {
    async fn evaluate_picure<T: std::fmt::Display + std::marker::Send>(
        &mut self,
        url: T,
    ) -> Result<Vec<(TagName, TagConfidence, TagClass)>, EvaluaterError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagClass {
    General,
    Artist,
    Unknown,
    Copyright,
    Character,
    Meta,
}

impl TagClass {
    pub fn from_num(num: u32) -> TagClass {
        match num {
            0 => TagClass::General,
            1 => TagClass::Artist,
            3 => TagClass::Copyright,
            4 => TagClass::Character,
            5 => TagClass::Meta,
            _ => TagClass::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvaluaterError {
    FailedCreatingEvaluater,
    FailedRequesting,
    FailedRetreiving,
    FailedOpening,
    FailedEvaluating(String),
}

pub struct ChildEvaluater {
    child: Child,
    tag_map: BTreeMap<u32, (TagName, TagClass)>,
}

/// Use a (blocking) child process as the evaluater
impl ChildEvaluater {
    /// Create a new std child process that serve as a evaluater
    pub async fn new() -> Result<ChildEvaluater, EvaluaterError> {
        let evaluater = Command::new("./eval_pic")
            .current_dir("./eval")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();
        if evaluater.is_err() {
            return Err(EvaluaterError::FailedCreatingEvaluater);
        }

        let mut evaluater = evaluater.unwrap();

        let mut reader = BufReader::new(evaluater.stdout.as_mut().unwrap());
        // Wait until the evaluater is ready, evaluater may spit out any nonsense before this message
        loop {
            let mut read_buf = String::new();
            if let Err(_) = reader.read_line(&mut read_buf) {
                return Err(EvaluaterError::FailedCreatingEvaluater);
            }
            if read_buf == "---ready---\n" {
                break;
            }
        }

        // Query information about the tags that the evaluater produce, and store the information in a BTreeMap
        let mut tag_map = std::collections::BTreeMap::new();
        if let Err(_) = writeln!(evaluater.stdin.as_ref().unwrap(), "tags") {
            return Err(EvaluaterError::FailedCreatingEvaluater);
        }
        if let Err(_) = evaluater.stdin.as_ref().unwrap().flush() {
            return Err(EvaluaterError::FailedCreatingEvaluater);
        }

        let mut read_buf = String::new();
        if let Err(_) = reader.read_line(&mut read_buf) {
            return Err(EvaluaterError::FailedCreatingEvaluater);
        }

        let line_count = match read_buf.trim().parse::<i32>() {
            Ok(num) => num,
            Err(_) => return Err(EvaluaterError::FailedCreatingEvaluater),
        };

        for _ in 0..line_count {
            read_buf.clear();
            if let Err(_) = reader.read_line(&mut read_buf) {
                return Err(EvaluaterError::FailedCreatingEvaluater);
            }
            // print!("{read_buf}");
            let tag = read_buf.trim().split(' ').collect::<Vec<&str>>();

            if tag.len() != 3 {
                return Err(EvaluaterError::FailedCreatingEvaluater);
            }
            let tag_num = match tag[0].parse::<u32>() {
                Ok(num) => num,
                Err(_) => return Err(EvaluaterError::FailedCreatingEvaluater),
            };

            let tag_classnum = match tag[2].parse::<u32>() {
                Ok(num) => num,
                Err(_) => return Err(EvaluaterError::FailedCreatingEvaluater),
            };

            tag_map.insert(
                tag_num,
                (tag[1].to_string(), TagClass::from_num(tag_classnum)),
            );
        }

        Ok(ChildEvaluater {
            child: evaluater,
            tag_map: tag_map,
        })
    }

    fn errorno_to_error(erronno: i32) -> EvaluaterError {
        match erronno {
            -1 => EvaluaterError::FailedRequesting,
            -2 => EvaluaterError::FailedRetreiving,
            -3 => EvaluaterError::FailedOpening,
            _ => EvaluaterError::FailedEvaluating(format!("Unknown Error {}", erronno)),
        }
    }
}

#[async_trait]
impl Evaluater for ChildEvaluater {
    async fn evaluate_picure<T: std::fmt::Display + std::marker::Send>(
        &mut self,
        url: T,
    ) -> Result<Vec<(TagName, TagConfidence, TagClass)>, EvaluaterError> {
        let evaluater = &mut self.child;
        if let Err(err) = writeln!(evaluater.stdin.as_ref().unwrap(), "{}", url) {
            return Err(EvaluaterError::FailedEvaluating(format!(
                "Error sending requesting child: {}",
                err.to_string()
            )));
        }
        if let Err(err) = evaluater.stdin.as_ref().unwrap().flush() {
            return Err(EvaluaterError::FailedEvaluating(format!(
                "Error sending requesting child: {}",
                err.to_string()
            )));
        }

        let mut reader = BufReader::new(evaluater.stdout.as_mut().unwrap());
        let mut read_buf = String::new();

        if let Err(err) = reader.read_line(&mut read_buf) {
            return Err(EvaluaterError::FailedEvaluating(format!(
                "Error reading requested data from child: {}",
                err.to_string()
            )));
        }

        let line_count = match read_buf.trim().parse::<i32>() {
            Ok(num) => num,
            Err(err) => {
                return Err(EvaluaterError::FailedEvaluating(format!(
                    "Error parsing requested data from child: {}",
                    err.to_string()
                )))
            }
        };

        // println!("{line_count}");
        if line_count >= 0 {
            let mut result = Vec::new();
            for _ in 0..line_count {
                read_buf.clear();
                if let Err(err) = reader.read_line(&mut read_buf) {
                    return Err(EvaluaterError::FailedEvaluating(format!(
                        "Error reading requested data from child: {}",
                        err.to_string()
                    )));
                }
                // print!("{read_buf}");
                let tag = read_buf.trim().split(' ').collect::<Vec<&str>>();
                if tag.len() != 2 {
                    return Err(EvaluaterError::FailedEvaluating(format!(
                        "Error parsing requested data from child: Incorrect format, too many value"
                    )));
                }
                let tag_num = match tag[0].parse::<u32>() {
                    Ok(num) => num,
                    Err(err) => {
                        return Err(EvaluaterError::FailedEvaluating(format!(
                            "Error parsing requested data from child: {}",
                            err.to_string()
                        )))
                    }
                };

                let confidence = match tag[1].parse::<TagConfidence>() {
                    Ok(num) => num,
                    Err(err) => {
                        return Err(EvaluaterError::FailedEvaluating(format!(
                            "Error parsing requested data from child: {}",
                            err.to_string()
                        )))
                    }
                };

                let (tag_name, tag_class) = match self.tag_map.get(&tag_num) {
                    Some(string) => string.clone(),
                    None => (format!("(Unknown Tag {}", tag_num), TagClass::Unknown),
                };

                result.push((tag_name, confidence, tag_class));
            }
            return Ok(result);
        } else {
            return Err(Self::errorno_to_error(line_count));
        }
    }
}
