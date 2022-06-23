extern crate exitcode;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use regex::Regex;
use std::fmt;

struct Section {
    name: String,
    start: Start,
    machine: Vec<u16>,
}

impl std::fmt::Debug for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = match self.start {
            Start::abs(st) => format!("{:#010x}", st),
            Start::rel(st) => format!("#{}", st),
        };
        write!(f, "Section - Name: {}, Start: {}", self.name.as_str(), start)
    }
}

enum Start {
    abs(u32),
    rel(usize),
}

impl Section {
    fn len(&self) -> u32 {
        self.machine.len() as u32
    }
}

fn assembler_error(reason: &str, line: &str) -> ! {
    println!("AASM Interpreter Error: {}\n\t{}", reason, line);
    std::process::exit(1);

}


fn main() {

    // READ FILE TO STRING
    let mut code_str = String::new();

    let path_str = match std::env::args().nth(1) { // get file name from args
        Some(file_path) => file_path,
        None => {
            println!("AASM: No file was given, aborting.");
            std::process::exit(exitcode::USAGE);
        }
    };

    let path = Path::new(&path_str);

    let mut file = match File::open(&path) { // open file
        Err(e) => {
            println!("AASM: Could not open {}: {}", path.display(), e);
            std::process::exit(exitcode::IOERR);
        },
        Ok(f) => f,
    };

    match file.read_to_string(&mut code_str) { // read file to string
        Err(e) => {
            println!("AASM: Could not read {}: {}", path.display(), e);
            std::process::exit(exitcode::IOERR);
        },
        Ok(_) => {},
    };

    //println!("{}", code_str);

    // SPLIT STRING INTO LINES

    let lines = code_str.lines();






    // START OF THE SMART PART

    let ram: Vec<u16> = Vec::new();

    let mut sections: Vec<Section> = Vec::new();

    let sec_re = Regex::new(r"\.(\w+):").unwrap();


    let mut start_sec = Section {
        name: String::from("start"),
        start: Start::abs(0x0000_0000),
        machine: Vec::new(),
    };

    let mut current_sec = &start_sec;

    sections.push(*current_sec);

    for l in lines {
        let mut words = l.split_whitespace();

        let first_word = match words.next() {
            Some(w) => w,
            None => continue,
        };


        if first_word.starts_with("//") { // ignore comments
            continue;

        } else if first_word.starts_with(".") { // sections

            let section_name = sec_re
                .captures(first_word)
                .unwrap_or_else(|| assembler_error("Incorrect format for start of section.", l))
                .get(1)
                .unwrap_or_else(|| assembler_error("Incorrect format for start of section.", l))
                .as_str();

            current_sec = match sections.iter().find(|s: &&Section| s.name == section_name) {
                Some(s) => s,
                None => {
                    let s = Section {
                        name: String::from(section_name),
                        start: Start::rel(sections.len()),
                        machine: Vec::new(),
                    };

                    sections.push(s);

                    &s
                }
            };

            println!("{:?}", sections);





        } else if words.next() == Some("LOADIMM") {
            println!("Hi");
        }

    }



}
