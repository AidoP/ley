use std::{fmt::Display, fs::File, io::{self, Write}, path::PathBuf};
use std::collections::VecDeque;

use crate::ley::*;
use super::catch;

pub struct Page {
    pub location: std::string::String,
    pub title: std::string::String
}

pub trait Format<'a>: Display + From<Ley<'a>> + std::ops::Deref<Target=Ley<'a>> {
    const EXTENSION: &'static str;
    fn index(target: PathBuf, pages: &'a [Page]) -> Option<&'static str> {
        let mut ley = Ley {
            title: Some("Index".to_string()).into(),
            author: None.into(),
            date: None.into(),
            style: None.into(),
            lines: LeyLines(vec![
                LeyLine::Section {
                    name: Some(String::new("Index")),
                    kind: SectionKind::Section,
                    contents: LeyLines(vec![])
                }
            ])
        };
        for page in pages {
            ley.lines.push(
                LeyLine::Section {
                    name: None,
                    kind: SectionKind::Paragraph,
                    contents: LeyLines(vec![
                        LeyLine::Section {
                            name: Some(String::new(page.location.as_str())),
                            kind: SectionKind::Link,
                            contents: LeyLines(vec![
                                LeyLine::Text {
                                    contents: String::new(page.title.as_str())
                                }
                            ])
                        }
                    ])
                }
            );
        }
        if let Err(error) = Self::from(ley).render("index", target) {
            Some(error)
        } else {
            None
        }
    }
    fn render(&self, name: &str, target: PathBuf) -> Result<Page, &'static str> {
        let mut file_name = name.to_string();
        file_name.push('.');
        file_name.push_str(Self::EXTENSION);
        let mut ley_destination = target.to_path_buf();
        ley_destination.push(file_name.clone());
        let mut ley_destination = if let Ok(file) = File::create(ley_destination.clone()) {
            file
        } else {
            eprintln!("Unable to create `{:?}`", ley_destination);
            return Err("File could not be opened with write permissions")
        };

        if write!(ley_destination, "{}", self).is_err() {
            Err("Unable to write to destination file")
        } else {
            Ok(Page {
                location: file_name,
                title: self.title.default("Untitled").to_string()
            })
        }
    }
}