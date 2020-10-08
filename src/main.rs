#![feature(try_trait)]

use std::{collections::VecDeque, env, fs::{File, read_dir}, io::{Read, Write}, path::Path};

pub mod ley;
pub use ley::{Ley, LeyLine, LeyLines};
mod fmt;
pub use fmt::{Format, Page};
mod html;
use html::Html;

fn main() {
    if let Some(error) = main_catch() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

#[macro_export]
macro_rules! catch {
    (some $message:expr => $value:expr) => {
        if let Some(value) = $value {
            value
        } else {
            return Some($message)
        }
    };
    (none $value:expr) => {
        if let value @ Some(_) = $value {
            return value
        }
    };
    (err $message:expr => $value:expr) => {
        if let None = $value {
            value
        } else {
            return Some($message)
        }
    };
    ($message:expr => $value:expr) => {
        if let Ok(value) = $value {
            value
        } else {
            return Some($message)
        }
    };
}

fn main_catch() -> Option<&'static str> {
    let mut index = false;
    let mut style = None;
    let mut ley_source = None;
    let mut ley_destination = None;

    {
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--style" => style = Some(catch!(some "style option requires an argument" => args.next())),
                "--index" => index = true,
                arg if arg.starts_with("--") => {
                    eprintln!("Unexpected argument `{}`", arg);
                    return Some("Unknown option")
                },
                arg => {
                    if ley_source.is_none() {
                        ley_source = Some(arg.to_string())
                    } else if ley_destination.is_none() {
                        ley_destination = Some(arg.to_string())
                    } else {
                        return Some("Too many positional arguments")
                    }
                }
            }
        }
    }
    let ley_source = catch!(some "A path to the file or directory to parse is required" => ley_source);
    let ley_source = Path::new(&ley_source);
    let ley_destination = ley_destination.unwrap_or(".".into());
    let ley_destination = Path::new(&ley_destination);

    if ley_source.is_dir() {
        if !ley_destination.is_dir() {
            return Some("The destination path must be a directory if the source path is a directory")
        }
        let mut pages = vec![];
        for source_path in catch!("Unable to read the source directory" => read_dir(ley_source)) {
            let source_path = catch!("Failed to iterate through directory" => source_path);
            if catch!("Unable to get file information" => source_path.file_type()).is_file() {
                let file_name = catch!("Ley does not currently support non-utf8 file names in the source directory" => source_path.file_name().into_string());
                if let Some(file_name) = file_name.strip_suffix(".ley") {
                    let mut ley_source = catch!("Unable to open source file" => File::open(source_path.path()));

                    let mut ley_contents = String::new();
                    catch!("Unable to read from ley file" => ley_source.read_to_string(&mut ley_contents));
                    match Ley::new(&ley_contents, style.clone().into()) {
                        Ok(ley) => {
                            pages.push(catch!("" => Html(ley).render(file_name, ley_destination.to_path_buf())))
                        }
                        Err(error) => {
                            eprintln!("Failed to parse Ley file. {}", error);
                            return Some("Errors in ley file")
                        }
                    }
                }
            }
        }
        if index {
            catch!(none Html::index(ley_destination.to_path_buf(), &pages, style.into()))
        }
        None
    } else if ley_source.is_file() {
        let mut ley_source = catch!("Unable to open source file" => File::open(ley_source));
        let mut ley_destination = catch!("Unable to create destination file" => File::create(ley_destination));
        let mut ley_contents = String::new();
        catch!("Unable to read from specified file" => ley_source.read_to_string(&mut ley_contents));
        match Ley::new(&ley_contents, style.into()) {
            Ok(ley) => {
                catch!("Unable to write to destination file" => write!(ley_destination, "{}", Html(ley)));
                None
            }
            Err(error) => {
                eprintln!("Failed to parse Ley file. {}", error);
                Some("Errors in ley file")
            }
        }
    } else {
        Some("The source path is invalid")
    }
}