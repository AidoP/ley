#![feature(try_trait)]

use std::{env, fs::File, io::{Read, Write}, path::Path};

pub mod ley;
pub use ley::{Ley, LeyLine, LeyLines};
mod fmt;

fn main() {
    if let Some(error) = main_catch() {
        eprintln!("Error: {}", error);
    }
}

macro_rules! catch {
    (some $message:expr => $value:expr) => {
        if let Some(value) = $value {
            value
        } else {
            return Some($message)
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
    let ley_path = catch!(some "A path to the file to parse is required" => env::args().nth(1));
    let mut ley_file = catch!("Unable to open specified file" => File::open(ley_path));
    let mut ley_contents = String::new();
    catch!("Unable to read from specified file" => ley_file.read_to_string(&mut ley_contents));
    let ley = match Ley::new(&ley_contents) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Failed to parse Ley file. {}", error);
            return Some("Errors in ley file")
        }
    };
    println!("{}", fmt::Html(ley));

    None
}