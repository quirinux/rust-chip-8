use std::path::PathBuf;
use structopt::StructOpt;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate log;
extern crate env_logger;
use env_logger::Env;

mod chip8;

#[derive(StructOpt, Debug)]
#[structopt(name = "chip8", about = "chip8 emulator, have funn !!!")]
struct Opt {
    /// Log level, will increase log level if passed multiple times: error, warn, info, debug, trace
    #[structopt(long, short, parse(from_occurrences))]
    debug: usize,

    /// rom to emulate 
    #[structopt(parse(from_os_str))]
    rom: PathBuf,

}

fn log_level(lvl: usize) -> String {
    let levels = ["none", "error", "warn", "info", "debug", "trace"];
    return if lvl >= levels.len(){
        levels[levels.len() - 1].to_string()
    } else {
        levels[lvl].to_string()
    };
}

fn main() {
    let opt = Opt::from_args();
    env_logger::from_env(Env::default().default_filter_or(log_level(opt.debug))).init();
    debug!("{:?}", opt); 
    let mut file = match File::open(opt.rom) {
        Ok(f) => f,
        Err(e) => {
            error!("chip8 - couldn't open the rom: {}", e);
            return
        },
    };
    let mut program_buffer = Vec::<u8>::new();
    match file.read_to_end(&mut program_buffer) {
        Err(e) => {
            error!("chip8 - couldn't load rom: {}", e);
            return
        },
        _ => {},
    };


    trace!("{:?}", program_buffer);

    let mut chip8 = match chip8::cpu::initialize() {
        Ok(cpu) => cpu,
        Err(e) => {
            error!("An error ocourred: {}", e);
            return
        }
    };
    
    chip8.bootup(program_buffer);

    chip8.run();
}
