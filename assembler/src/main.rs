extern crate exitcode;
extern crate hex;

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
        write!(f, "Section - Name: {}, Start: {}, Values: {:?}", self.name.as_str(), start, self.machine)
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
    println!("AASM Interpreter Error: {}\n\t\"{}\" lmao", reason, line);
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

    let ram: Vec<u16> = Vec::new(); //

    /*
    HOW THIS WORKS:

    1. Gather sections
    2. Machine code is assembled inside the sections
    3. Combine sections one after another
    4. Get memory addresses for labels in RAM
    */



    // REGEXS
    let sec_re = Regex::new(r"\.(\w+)(?: +0x([0-9a-fA-F_]+))?:").unwrap();

    let re_imm_values = Regex::new(r"(^|\s)#(\d+),?(\s|$)").unwrap();
    let re_thin_registers = Regex::new(
        r"(?:\sR(\d{1,2}),?(?:\s|$))|\s([ABCDEFGH])|(?:([PJKL]|PC).?([01])),?(?:\s|$)")
        .unwrap();
    let re_wide_registers = Regex::new(r"(?:^|\s)([PJKL]|PC),?(?:\s|$)")
        .unwrap();



    // SECTIONS
    let mut sections: Vec<Section> = Vec::new();

    sections.push(
        Section { // if it's start section, set that up with some defaults
            name: String::from("start"),
            start: Start::abs(0x0000_0000),
            machine: Vec::new(),
        }
    );

    let mut current_sec = sections.get_mut(0).unwrap();


    // ERROR MESSAGES
    const sections_error_message: &str = "Incorrect format for start of section. Correct \
        formats include:\n  .[section_name]:    .[section_name] 0x[ram_loc_hex]]:";

    // LETS GET ASSEMBLING
    for l in lines {
        let mut words = l.split_whitespace();

        let first_word = match words.next() {
            Some(w) => w,
            None => continue, // skips whitespace lines
        };


        if first_word.starts_with("//") { // ignore comments
            continue;

        } else if first_word.starts_with(".") { // sections



            let section_name = sec_re
                .captures(l)
                .unwrap_or_else(|| assembler_error(sections_error_message, l))
                .get(1)
                .unwrap_or_else(|| assembler_error(sections_error_message, l))
                .as_str();

            let ram_loc_hex = sec_re // this is an Option<Match>
                .captures(l)
                .unwrap_or_else(|| assembler_error(sections_error_message, l))
                .get(2);

            current_sec = match sections.iter_mut().find(|s: &&mut Section| s.name == section_name) {
                Some(s) => s, // if section already in our list, give us the reference to it
                None => { // otherwise, make it

                    let s = match section_name {
                        "start" => Section { // if it's start section, set that up with some defaults
                            name: String::from("start"),
                            start: Start::abs(0x0000_0000),
                            machine: Vec::new(),
                        },
                        other => {
                            match ram_loc_hex {
                                Some(m) => {
                                    Section {
                                        name: String::from(section_name),
                                        start: Start::abs( // parse hex value as we assign it to start
                                            u32::from_str_radix(&m.as_str().replace("_", ""), 16)
                                                .unwrap_or_else(|_| assembler_error("Invalid address value for section", l))
                                        ),
                                        machine: Vec::new(),
                                    }
                                },
                                None => {
                                    Section {
                                        name: String::from(section_name),
                                        start: Start::rel(sections.len()),
                                        machine: Vec::new(),
                                    }
                                }
                            }
                        }
                    };


                    sections.push(s);

                    sections.last_mut().unwrap() // return last e

                }
            };


            //println!("{:?}", sections);
            //println!("{:?}", current_sec);


        } else {


            // literally just find tokens in the string

            let imm_values_list: Vec<u16> = re_imm_values
                .captures_iter(l)
                .map(|x| u16::from_str_radix(x.get(2).unwrap().as_str(), 10)
                    .unwrap_or_else(|_| assembler_error(
                        &format!("Invalid immediate value. Not sure what \"{}\" is",
                            x.get(0).unwrap().as_str()), l)
                )).collect();

            for i in &imm_values_list {
                if *i >= 512 {
                    assembler_error(&format!("Immediate value too large. Architecture only allows \
                    9 bit immediates within LOADIMM and 6 bit immediates elsewhere. Your immediate \
                    was {}", i), l);
                } else if *i >= 64 && first_word != "LOADIMM" {
                    assembler_error(&format!("Immediate value too large. Architecture only allows 6 \
                    bit immediates outside of LOADIMM. Your immediate was {}", i), l);

                }
            }

            let thin_register_list: Vec<u16> = re_thin_registers //TODO: Include R numbers in possibilities
                .captures_iter(l)
                .map(|x| match x.get(0).unwrap().as_str().chars().nth(1).unwrap() {
                    'A' => 0b0000u16,
                    'B' => 0b0001u16,
                    'C' => 0b0010u16,
                    'D' => 0b0011u16,
                    'E' => 0b0100u16,
                    'F' => 0b0101u16,
                    'G' => 0b0110u16,
                    'H' => 0b0111u16,
                    'J' => 0b1000u16 +
                            x
                            .get(4).unwrap().as_str()
                            .chars()
                            .next().unwrap()
                            .to_digit(10).unwrap()
                            as u16,
                    'K' => 0b1010u16 +
                            x
                            .get(4).unwrap().as_str()
                            .chars()
                            .next().unwrap()
                            .to_digit(10).unwrap()
                            as u16,
                    'L' => 0b1100u16 +
                            x
                            .get(4).unwrap().as_str()
                            .chars()
                            .next().unwrap()
                            .to_digit(10).unwrap()
                            as u16,
                    'P' => 0b1110u16 +
                            x
                            .get(4).unwrap().as_str()
                            .chars()
                            .next().unwrap()
                            .to_digit(10).unwrap()
                            as u16,

                    'R' => x.get(1).unwrap().as_str().parse::<u16>().unwrap(),

                     e => panic!("Unexpected item in bagging area: {}. (this should be an unreachable state.)", x.get(0).unwrap().as_str())
                })
                .collect();

            let wide_register_list: Vec<u16> = re_wide_registers
                .captures_iter(l)
                .map(|x| match x.get(1).unwrap().as_str().chars().next().unwrap() {
                    'J' => 0b00u16,
                    'K' => 0b01u16,
                    'L' => 0b10u16,
                    'P' => 0b11u16,
                     _  => panic!("Unexpected item in bagging area. (this should be an unreachable state.)")
                })
                .collect();



            match first_word {
                "LOADIMM" => {

                    if imm_values_list.len() == 1 && thin_register_list.len() == 1 {
                        let imm = imm_values_list.get(0).unwrap();
                        let reg = thin_register_list.get(0).unwrap();

                        let machine: u16 = 0b0100_0000_0000_0000 | imm << 3; // uses 9 bit immediates
                        current_sec.machine.push(machine);

                    } else {
                        assembler_error(
                            "Pattern doesn't match expected pattern for LOADIMM. Expected pattern \
                            is:\n  LOADIMM #[imm_value], [thin_register]", l);
                    }

                },
                &_ => todo!("Unrecognized word.")
            }
        }


    }

    println!("{:?}", sections);



}
