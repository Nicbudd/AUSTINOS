#[allow(non_snake_case)]

pub mod emulator;

use std::env::args;
use std::io::*;
use std::fs::File;
use std::iter::zip;


fn pr(statement: &str) {
    println!("AEMU>>> {}", statement);
}

fn start_emulator(ram_path: &str){

    let mut buff = Vec::new();

    let mut f = match File::open(ram_path) {
        Ok(file) => file,
        Err(_) => {
            pr("Failed to open file");
            return;
        },
    };

    match f.read_to_end(&mut buff) {
        Err(_) => {
            pr("Failed to read file");
            return;
        },
        Ok(_) => {},
    };

    let ram: Vec<u16> = zip(
        buff.iter().skip(0).step_by(2),
        buff.iter().skip(1).step_by(2),
    ).map(|(x, y)| ((*x as u16) << 8) + (*y as u16)).collect();

    //println!("{:?}", ram);

    pr("Starting AustinOS Emulator...");

    emulator::run(ram);
}


fn main() {

    let path = args().nth(1);

    match path {
        Some(p) => {
            start_emulator(&p);
        },
        None => {
            pr("Please provide a binary file (.abin) to run.");
            std::process::exit(1);
        }
    };


}
