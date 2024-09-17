//! A CLI for the `chip8` library
//!
//! Handles loading programs, argument parsing and storing sound data

#[macro_use]
extern crate error_chain;
extern crate env_logger;
extern crate app_dirs;
extern crate chip8;
extern crate clap;

mod sound;
mod load;

use clap::{App, Arg};
use chip8::default_io::Io;

quick_main!(run);

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

/// Loads a program from a file and runs in it a Chip-8 emulator
fn run() -> chip8::Result<()> {
    env_logger::init().unwrap();

    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about("A Chip-8 emulator")
        .arg(Arg::with_name("file").required(true))
        .arg(Arg::with_name("log")
            .short("l")
            .long("enable-logging")
            .help("Enable logging of opcodes"))
        .get_matches();

    let log = matches.is_present("log").into();
    let file = matches.value_of("file").unwrap();
    let program = load::load_program(file).unwrap_or_else(|e| {
        panic!("Could not load program from file: `{}` ({})", file, e);
    });

    // Get the path to the sound file
    let sound_path = sound::sound_path();
    // Initialize I/O state
    let mut io = Io::new(&sound_path);

    chip8::run(&program, &mut io, log)
}
