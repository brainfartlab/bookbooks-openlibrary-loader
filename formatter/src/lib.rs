use std::error::Error;
use std::fs::File;
use std::io;

use csv::{
    QuoteStyle,
    Reader,
    ReaderBuilder,
    Result as CsvResult,
    Writer,
    WriterBuilder,
};
use serde::{Deserialize, Serialize};

pub enum Task {
    Format(String),
    Assign(String),
}

pub struct Config {
    //delimiter: char,
    pub task: Task,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Record {
    //created: Created,
    #[serde(flatten)]
    entity: Entity,
    revision: u32,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase", untagged)]
enum Entity {
    Author {
        name: String,
        personal_name: Option<String>,
    },
    Work {
        authors: Option<Vec<AuthorReference>>,
        subtitle: Option<String>,
        title: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
struct AuthorReference {
    author: AuthorKey,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct AuthorKey {
    key: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Created {
    r#type: String,
    value: String,
}

impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Self, &'static str> {
        args.next();

        if let Some(operator) = args.next() {
            let task = match operator.as_str() {
                "assign" => {
                    let filename = match args.next() {
                        Some(arg) => arg,
                        None => return Err("no filename supplied"),
                    };

                    Task::Assign(filename)
                },
                "format" => {
                    let filename = match args.next() {
                        Some(arg) => arg,
                        None => return Err("no filename supplied"),
                    };

                    Task::Format(filename)
                },
                _ => {
                    panic!("unrecognized operation");
                },
            };

            return Ok(Self {
                //delimiter,
                task,
            })
        } else {
            return Err("no operator specified");
        }

        //let delimiter = match args.next() {
        //    Some(arg) => arg.chars().next().unwrap(),
        //    None => '|'
        //};

    }
}

fn clean_string(arg: &str) -> String {
    let arg = arg.replace("\n", "");
    let arg = arg.replace("\\", "");

    arg
}

fn create_reader(filename: &str) -> CsvResult<Reader<File>> {
    let rdr = ReaderBuilder::new()
        .has_headers(false)
        .flexible(false)
        .delimiter(b'\t')
        .from_path(filename);

    return rdr;
}

fn create_writer() -> Writer<io::Stdout> {
    let wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .quote(b'"')
        .double_quote(false)
        .quote_style(QuoteStyle::Necessary)
        .from_writer(io::stdout());

    return wtr;
}

pub fn assign(filename: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = create_reader(filename)?;
    let mut wtr = create_writer();

    for result in rdr.records() {
        let record = result?;

        let key = &record[1];
        let data: Record = match serde_json::from_str(&record[4]) {
            Ok(data) => data,
            Err(_err) => {
                eprintln!("could not process work: {key}");
                continue;
            },
        };

        match data.entity {
            Entity::Work { authors, subtitle: _, title: _ } => {
                match authors {
                    Some(authors) => {
                        authors.iter()
                            .for_each(|author| {
                                wtr.write_record(&[
                                    key,
                                    &author.author.key,
                                ]).unwrap();
                            });
                    },
                    None => (),
                }
            }
            _ => (),
        }
    }

    wtr.flush()?;

    Ok(())
}

pub fn format(filename: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = create_reader(filename)?;
    let mut wtr = create_writer();

    for result in rdr.records() {
        let record = result?;

        let key = &record[1];
        let revision = &record[2];
        let last_modified = &record[3];

        let data: Record = match serde_json::from_str(&record[4]) {
            Ok(data) => data,
            Err(_err) => {
                eprintln!("could not deserialize: {:?}", &record[4]);
                continue;
            },
        };

        match data.entity {
            Entity::Author { name, personal_name } => {
                let name = clean_string(&name);
                let personal_name = clean_string(&personal_name.unwrap_or(String::from("")));

                wtr.write_record(&[
                    key,
                    revision,
                    last_modified,
                    &name,
                    &personal_name,
                    //&data.created.value,
                ])?;
            },
            Entity::Work { authors: _, subtitle, title } => {
                let title = clean_string(&title);
                let subtitle = clean_string(&subtitle.unwrap_or(String::from("")));

                wtr.write_record(&[
                    key,
                    revision,
                    last_modified,
                    &title,
                    &subtitle,
                    //&data.created.value,
                ])?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn serialize_author_record() {
        let author = Record {
            //created: Created {
            //    r#type: String::from("/type/datetime"),
            //    value: String::from("2008-04-01T03:28:50.625462"),
            //},
            entity: Entity::Author {
                name: String::from("name"),
                personal_name: Some(String::from("personal name")),
            },
            revision: 1,
        };

        let author_json = json!({
            "name": "name",
            "personal_name": "personal name",
            //"created": {
            //    "type": "/type/datetime",
            //    "value": "2008-04-01T03:28:50.625462",
            //},
            "revision": 1,
        });

        let serialized_author_result = serde_json::to_string(&author);
        assert!(serialized_author_result.is_ok());

        let serialized_author = serde_json::to_value(&author).unwrap();
        assert_eq!(serialized_author, author_json);
    }

    #[test]
    fn deserialize_author_record() {
        let json = r#"
            {
                "name": "name",
                "personal_name": "personal name",
                "revision": 1
            }
        "#;

        let record: Record = serde_json::from_str(&json).unwrap();

        assert_eq!(record, Record {
            //created: Created {
            //    r#type: String::from("/type/datetime"),
            //    value: String::from("2008-04-01T03:28:50.625462"),
            //},
            entity: Entity::Author {
                name: String::from("name"),
                personal_name: Some(String::from("personal name")),
            },
            revision: 1,
        });
    }

    #[test]
    fn serialize_work_record() {
        let work = Record {
            entity: Entity::Work {
                authors: Some(vec![
                    AuthorReference {
                        author: AuthorKey {
                            key: String::from("/authors/1"),
                        },
                    },
                    AuthorReference {
                        author: AuthorKey {
                            key: String::from("/authors/2"),
                        },
                    },
                ]),
                title: String::from("title"),
                subtitle: Some(String::from("subtitle")),
            },
            //created: Created {
            //    r#type: String::from("/type/datetime"),
            //    value: String::from("2008-04-01T03:28:50.625462"),
            //},
            revision: 1,
        };

        let work_json = json!({
            "authors": [
                {
                    "author": {
                        "key": "/authors/1",
                    },
                },
                {
                    "author": {
                        "key": "/authors/2",
                    },
                },
            ],
            //"created": {
            //    "type": "/type/datetime",
            //    "value": "2008-04-01T03:28:50.625462",
            //},
            "subtitle": "subtitle",
            "title": "title",
            "revision": 1,
        });

        let serialized_work_result = serde_json::to_string(&work);
        assert!(serialized_work_result.is_ok());

        let serialized_work = serde_json::to_value(&work).unwrap();
        assert_eq!(serialized_work, work_json);
    }

    #[test]
    fn deserialize_work_record() {
        let json = r#"
            {
                "authors": [
                    {
                        "author": {
                            "key": "/authors/1"
                        }
                    },
                    {
                        "author": {
                            "key": "/authors/2"
                        }
                    }
                ],
                "subtitle": "subtitle",
                "title": "title",
                "revision": 1
            }
        "#;

        let record: Record = serde_json::from_str(&json).unwrap();

        assert_eq!(record, Record {
            entity: Entity::Work {
                authors: Some(vec![
                    AuthorReference {
                        author: AuthorKey {
                            key: String::from("/authors/1"),
                        },
                    },
                    AuthorReference {
                        author: AuthorKey {
                            key: String::from("/authors/2"),
                        },
                    },
                ]),
                title: String::from("title"),
                subtitle: Some(String::from("subtitle")),
            },
            //created: Created {
            //    r#type: String::from("/type/datetime"),
            //    value: String::from("2008-04-01T03:28:50.625462"),
            //},
            revision: 1,
        });
    }
}
