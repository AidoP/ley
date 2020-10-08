use crate::{Ley, LeyLine, LeyLines, ley};

use std::fmt::{self, Display, Formatter};

pub struct Html<'a>(pub Ley<'a>);
impl<'a> Display for Html<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, include_str!("main.html"), title = self.0.title.as_ref().unwrap_or(&"Untitled Page".into()), content = InnerHtml(&self.0.lines, 1), author = self.0.author.as_ref().unwrap_or(&"No Author".into()), date = self.0.date.as_ref().unwrap_or(&"Unknown Date".into()), style = self.0.style.as_ref().unwrap_or(&"main.css".into()))
    }
}

struct InnerHtml<'a>(pub &'a LeyLines<'a>, usize);
impl<'a> Display for InnerHtml<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use LeyLine::*;
        for ley_line in self.0.iter() {
            use ley::SectionKind;
            match ley_line {
                Section { name: Some(name), contents, kind: SectionKind::Section} => write!(
                    f,
                    "<h{depth} id=\"{name}\">{name}</h{depth}><div class=\"depth_{depth}\">{contents}</div>",
                    name = name,
                    contents = InnerHtml(&contents, self.1 + 1),
                    depth = self.1
                )?,
                Section { contents, kind: SectionKind::Paragraph, ..} | Section { name: None, contents, kind: SectionKind::Section } => write!(f, "<p>{}</p>", InnerHtml(&contents, self.1))?,
                Text { contents } => write!(f, "{}", contents)?,
                Section { name: Some(name), contents, kind: SectionKind::Link } => write!(f, "<a href=\"{name}\">{contents}</a>", name = name, contents = InnerHtml(&contents, self.1))?,
                Section { contents, kind: SectionKind::Code, ..} => write!(f, "<code>{contents}</code>", contents = InnerHtml(&contents, self.1))?,
                Section { name: None, contents, kind: SectionKind::Link } => write!(f, "<a>{contents}</a>", contents = InnerHtml(&contents, self.1))?,
                Section { name: Some(name), kind: SectionKind::Image, ..} => write!(f, "<img src=\"{name}\">", name = name)?,
                Comment | Section { kind: SectionKind::Metadata, ..} | Section { kind: SectionKind::Image, ..} => ()
            }
        }
        Ok(())
    }
}