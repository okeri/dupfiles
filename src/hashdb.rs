use std::io;
use std::io::prelude::*;
use std::fs::{File, remove_file, rename, create_dir_all};
use std::path::Path;
use std::collections::BTreeMap;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use getch::Getch;

#[derive(Clone)]
enum Rule {
    Print,
    Rm(String),
    Move(String, String),
}

pub struct HashDB {
    db: BTreeMap<String, String>,
    rules: BTreeMap<String, Rule>,
    hasher: Sha256,
}

impl HashDB {
    fn getch(message: &str, accept: &str) -> u8 {
        let console = Getch::new();
        loop {
            println!("{}", message);
            if let Ok(v) = console.getch() {
                if accept.find(v as char).is_some() {
                    return v;
                }
            }
        }
    }
    fn query_dir() -> io::Result<String> {
        let mut result: String = "".to_string();
        while result.is_empty() {
            println!("What path we should use for backup files?");
            io::stdin().read_line(&mut result)?;
        }
        result.pop();
        create_dir_all(result.clone())?;
        Ok(result)
    }

    fn query_rule(path: &str, conflict: &str) -> Rule {
        let key = HashDB::getch("[P]rint, [D]elete, [M]ove?: ", "pdm");
        match key {
            100u8 => {
                let ask = format!(
                    "Which path is preferable for files?\n1) {}\n2) {}",
                    path,
                    conflict
                );
                match HashDB::getch(&ask, "12") {
                    50u8 => Rule::Rm(path.to_owned()),
                    _ => Rule::Rm(conflict.to_owned()),
                }
            }

            109u8 => {
                let ask = format!(
                    "Which path is preferable for files?\n1) {}\n2) {}",
                    path,
                    conflict
                );
                let keep = HashDB::getch(&ask, "12");
                if let Ok(dir) = HashDB::query_dir() {
                    match keep {
                        50u8 => Rule::Move(path.to_owned(), dir),
                        _ => Rule::Move(conflict.to_owned(), dir),
                    }
                } else {
                    Rule::Print
                }
            }

            _ => Rule::Print,
        }
    }

    fn rule(&mut self, path: &str, conflict: &str) -> Rule {
        if let Some(found) = self.rules.get(path) {
            found.clone()
        } else {
            println!("files in {} duplicates of {}", path, conflict);
            let rule = HashDB::query_rule(path, conflict);
            self.rules.insert(path.to_owned(), rule.clone());
            rule
        }
    }

    fn find(&self, filename: &str) -> Option<String> {
        self.db.get(filename).and_then(|v| Some(v.clone()))
    }

    fn hash(&mut self, filename: &Path) -> io::Result<String> {
        let mut input = File::open(filename)?;
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        self.hasher.reset();
        self.hasher.input(buffer.as_mut());
        Ok(self.hasher.result_str())
    }

    pub fn process_file(&mut self, path: &Path) {
        if let Some(filename) = path.to_str() {
            if let Ok(hash) = self.hash(path) {
                if let Some(found) = self.find(&hash) {
                    if let Some(parent) = path.parent() {
                        let found_parent = Path::new(&found).parent().unwrap();
                        let rule =
                            self.rule(parent.to_str().unwrap(), found_parent.to_str().unwrap());
                        match rule {
                            Rule::Print => {
                                println!("{} is duplicate of {}", filename, found);
                            }

                            Rule::Rm(p) => {
                                if p == parent.to_str().unwrap() {
                                    remove_file(path).unwrap_or_default();
                                } else {
                                    remove_file(found).unwrap_or_default();
                                }
                            }

                            Rule::Move(p, d) => {
                                let destdir = Path::new(&d);
                                if p == parent.to_str().unwrap() {
                                    let fname = path.file_name().unwrap();
                                    println!(
                                        "{} -> {}",
                                        path.to_str().unwrap(),
                                        destdir.join(fname).to_str().unwrap()
                                    );
                                    rename(path, destdir.join(fname)).unwrap_or_default();
                                } else {
                                    let fname = Path::new(&found).file_name().unwrap();
                                    println!(
                                        "{} -> {}",
                                        found,
                                        destdir.join(fname).to_str().unwrap()
                                    );
                                    rename(&found, destdir.join(fname)).unwrap_or_default();
                                }
                            }
                        }
                    }
                } else {
                    self.db.insert(hash, filename.to_owned());
                }
            }
        }
    }

    pub fn new() -> HashDB {
        HashDB {
            db: BTreeMap::new(),
            rules: BTreeMap::new(),
            hasher: Sha256::new(),
        }
    }
}
