use std::{cmp::PartialEq, fmt::{self, Debug, Display, Formatter}, ops::{Deref,DerefMut}};

/// A parsed ley file
pub struct Ley<'a> {
    pub lines: LeyLines<'a>,
    pub title: Metadata,
    pub author: Metadata,
    pub date: Metadata,
    pub style: Metadata
}
impl<'a> Ley<'a> {
    pub fn new(mut source: &'a str, style: Metadata) -> Result<Self, ParseError<'a>> {
        let mut token_stream = Vec::new();
        while let Some(token) = Token::parse(&mut source) {
            token_stream.push(token)
        }
        let mut token_stream = token_stream.as_slice();
        let mut token_stream = TokenIter(&mut token_stream);

        let mut lines = Vec::new();
        let (mut title, mut author, mut date, mut style) = (Metadata::NONE, Metadata::NONE, Metadata::NONE, style);
        while let Some(_) = token_stream.peek() {
            use LeyLine::*;
            match LeyLine::parse(&mut token_stream)? {
                Section { name: Some(name), contents, kind: SectionKind::Metadata } => {
                    if let Some(&name) = name.get(0) {
                        match name {
                            "title" => title = Metadata::from_lines(contents)?,
                            "author" => author = Metadata::from_lines(contents)?,
                            "date" => date = Metadata::from_lines(contents)?,
                            "style" => style = Metadata::from_lines(contents)?,
                            _ => eprintln!("Warning: Unknown Metadata {}", name)
                        }
                    }
                }
                ley_line => lines.push(ley_line)
            }
        }
        let lines = LeyLines(lines);
        
        Ok(Self {
            lines,
            title,
            author,
            date,
            style
        })
    }
}

pub struct Metadata(Option<std::string::String>);
impl Metadata {
    const NONE: Self = Self(None);
    pub fn from_lines<'a>(mut ley_lines: LeyLines<'a>) -> Result<Self, ParseError> {
        if ley_lines.len() == 1 {
            if let LeyLine::Text { contents } = ley_lines.remove(0) {
                Ok(Self(Some(format!("{}", contents))))
            } else {
                Err(ParseError::ExpectedString)
            }
        } else {
            Err(ParseError::ExpectedString)
        }
    }
}
impl Deref for Metadata {
    type Target = Option<std::string::String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Metadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Option<std::string::String>> for Metadata {
    fn from(from: Option<std::string::String>) -> Self {
        Self(from)
    }
}

#[derive(Debug)]
pub struct LeyLines<'a>(Vec<LeyLine<'a>>);
impl <'a> LeyLines<'a> {
    pub fn new(mut source: &'a str) -> Result<Self, ParseError> {
        let mut token_stream = Vec::new();
        while let Some(token) = Token::parse(&mut source) {
            token_stream.push(token)
        }
        let mut token_stream = token_stream.as_slice();
        let mut token_stream = TokenIter(&mut token_stream);

        let mut ley_lines = Vec::new();
        while let Some(_) = token_stream.peek() {
            ley_lines.push(LeyLine::parse(&mut token_stream)?)
        }
        
        Ok(Self(ley_lines))
    }
}
impl<'a> Deref for LeyLines<'a> {
    type Target = Vec<LeyLine<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> DerefMut for LeyLines<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A portion of a ley file
#[derive(Debug)]
pub enum LeyLine<'a> {
    Section {
        name: Option<String<'a>>,
        kind: SectionKind,
        contents: LeyLines<'a>
    },
    Text {
        contents: String<'a>
    },
    Comment
}
impl<'a> LeyLine<'a> {
    pub fn parse(token_stream: &mut TokenIter<'a, '_, '_>) -> Result<Self, ParseError<'a>> {
        use Token::*;
        match token_stream.next()? {
            Exclamation => {
                let name = String::<'a>::parse(token_stream);
                let comment = match token_stream.next()? {
                    Colon => false,
                    SemiColon => true,
                    _ => return Err(ParseError::ExpectedColon)
                };
                let kind = match token_stream.next()? {
                    Identifier(ident) => {
                        if *token_stream.next()? != Token::OpenBrace {
                            return Err(ParseError::ExpectedOpenBrace)
                        }
                        if comment {
                            SectionKind::Section
                        } else {
                            SectionKind::new(ident)?
                        }
                    },
                    OpenBrace => if name.is_none() { SectionKind::Paragraph } else { SectionKind::Section },
                    _ => return Err(ParseError::ExpectedOpenBrace)
                };
                let mut contents = vec![];
                while *token_stream.peek()? != CloseBrace {
                    contents.push(Self::parse(token_stream)?)
                }
                let _ = token_stream.next();
                Ok(
                    if comment {
                        Self::Comment
                    } else {
                        Self::Section {
                            name,
                            kind,
                            contents: LeyLines(contents)
                        }
                    }
                )
            }
            Identifier(ident) => {
                Ok(Self::Text {
                    contents: String::with(ident, token_stream)
                })
            }
            token => panic!("Unexpected {:?}, {:?}", token, token_stream)
        }
    }
}

#[derive(Debug)]
pub enum SectionKind {
    Section,
    Paragraph,
    Metadata,

    Link,
    Image,
    Code
}
impl SectionKind {
    pub fn new(from: &str) -> Result<Self, ParseError> {
        match from {
            "section" => Ok(Self::Section),
            "paragraph" | "para" | "p" => Ok(Self::Paragraph),
            "meta" | "metadata" => Ok(Self::Metadata),
            "link" => Ok(Self::Link),
            "image" | "img" => Ok(Self::Image),
            "code" | "lang" => Ok(Self::Code),
            kind => Err(ParseError::UnknownSection(kind))
        }
    }
}

#[derive(Clone, Debug)]
pub struct String<'a>(Vec<&'a str>);
impl<'a> String<'a> {
    pub fn new(string: &'a str) -> Self {
        Self(vec![string])
    }
    pub fn parse(tokens: &mut TokenIter<'a, '_, '_>) -> Option<Self> {
        let mut string = Self(vec![]);
        while let Some(Token::Identifier(ident)) = tokens.peek() {
            tokens.next();
            string.push(ident)
        }
        if string.len() != 0 {
            Some(string)
        } else {
            None
        }
    }
    pub fn with(string: &'a str, tokens: &mut TokenIter<'a, '_, '_>) -> Self {
        let mut string = Self(vec![string]);
        while let Some(Token::Identifier(ident)) = tokens.peek() {
            tokens.next();
            string.push(ident)
        }
        string
    }
    pub fn from_lines(mut ley_lines: LeyLines<'a>) -> Result<Self, ParseError> {
        if ley_lines.len() == 1 {
            if let LeyLine::Text { contents } = ley_lines.remove(0) {
                Ok(contents)
            } else {
                Err(ParseError::ExpectedString)
            }
        } else {
            Err(ParseError::ExpectedString)
        }
    }
}
impl<'a> Deref for String<'a> {
    type Target = Vec<&'a str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> DerefMut for String<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'a> Display for String<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut iter = self.iter();
        if let Some(ident) = iter.next() {
            write!(f, "{}", ident)?
        }
        for ident in iter {
            write!(f, " {}", ident)?
        }
        Ok(())
    }
}
impl<'a> PartialEq<&str> for String<'a> {
    fn eq(&self, other: &&str) -> bool {
        if let Some(&value) = self.0.get(0) {
            value == *other
        } else {
            false
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token<'a> {
    Identifier(&'a str),
    OpenBrace,
    CloseBrace,
    Exclamation,
    Colon,
    SemiColon,
    Star,
    DoubleStar,
    Backtick,
    Underscore,
    Tilde
}
impl<'a> Token<'a> {
    pub fn parse(from: &mut &'a str) -> Option<Self> {
        let mut ident = None;
        let mut pos = 0;
        let mut chars = from.chars();
        while let Some(c) = chars.next() {
            match c {
                ' ' | '\n' | '\r' | '\t' => if let Some(ident) = ident {
                    let to_return = Some(Self::Identifier(&from[ident..pos]));
                    *from = &from[pos+1..];
                    return to_return;
                }
                '{' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let to_return = Some(Self::OpenBrace);
                        *from = &from[pos+1..];
                        to_return
                    }
                }
                '}' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let to_return = Some(Self::CloseBrace);
                        *from = &from[pos+1..];
                        to_return
                    }
                }
                '!' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let to_return = Some(Self::Exclamation);
                        *from = &from[pos+1..];
                        to_return
                    }
                },
                ':' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let to_return = Some(Self::Colon);
                        *from = &from[pos+1..];
                        to_return
                    };
                },
                ';' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let to_return = Some(Self::SemiColon);
                        *from = &from[pos+1..];
                        to_return
                    };
                }
                '"' => {
                    return if let Some(ident) = ident {
                        let to_return = Some(Self::Identifier(&from[ident..pos]));
                        *from = &from[pos..];
                        to_return
                    } else {
                        let mut end = pos;
                        if chars.next()? == '"' {
                            loop {
                                end += 1;
                                if chars.next()? == '"' {
                                    end += 1;
                                    if chars.next()? == '"' {
                                        break
                                    }
                                }
                            }
                        } else {
                            while chars.next()? != '"' {
                                end += 1
                            }
                            end += 1
                        }
                        let to_return = Some(Self::Identifier(&from[pos+1..end+1]));
                        *from = &from[end+2..];
                        to_return
                    };
                }
                _ => if ident == None {
                    ident = Some(pos)
                }
            }
            pos += 1
        }
        None
    }
}
#[derive(Debug)]
pub struct TokenIter<'a, 'b, 'c>(&'c mut &'b [Token<'a>]);
impl<'a, 'b, 'c> TokenIter<'a, 'b, 'c> {
    pub fn peek<'d>(&'d self) -> Option<&'b Token<'a>> {
        self.0.get(0)
    }
}
impl<'a, 'b, 'c> Iterator for TokenIter<'a, 'b, 'c> {
    type Item = &'b Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.0.get(0) {
            *self.0 = &self.0[1..];
            Some(token)
        } else {
            None
        }
    }
}

pub enum ParseError<'a> {
    EndOfFile,
    UnclosedSection,
    UnexpectedCloseBracket,
    UnknownSection(&'a str),
    ExpectedColon,
    ExpectedOpenBrace,
    ExpectedString
}
impl<'a> From<std::option::NoneError> for ParseError<'a> {
    fn from(_: std::option::NoneError) -> Self {
        ParseError::EndOfFile
    }
}
impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfFile => write!(f, "Unexpected end of file"),
            Self::UnclosedSection => write!(f, "Unclosed Section, Expected `}}`"),
            Self::UnexpectedCloseBracket => write!(f, "Unexpected `}}`"),
            Self::UnknownSection(section) => write!(f, "Unknown Section Kind `{}`", section),
            Self::ExpectedColon => write!(f, "Expected `:`"),
            Self::ExpectedOpenBrace => write!(f, "Expected `{{`"),
            Self::ExpectedString => write!(f, "Expected a string")
        }
    }
}