use std::{thread, time};
//use std::io;
use super::optcodes::*;
use super::sound::*;
//use super::keyboard::*;

pub(crate) const STACK_SIZE: usize = 0xF + 1;
pub(crate) const MEMORY_SIZE: usize = 4_096;
pub(crate) const KEYBOARD_SIZE: usize = 0xF + 1;

// chip-8 original resolution
pub(crate) const SCREEN_WIDTH: u32 = 64;
pub(crate) const SCREEN_HEIGHT: u32 = 32;
pub(crate) const WAIT_INTERVAL: std::time::Duration = time::Duration::from_millis(1000 / 60); // 60hrz

// super-8 resolution
// pub(crate) const SCREEN_WIDTH: u32 = 128;
// pub(crate) const SCREEN_HEIGHT: u32 = 64;

pub struct Cpu {
    pub(crate) v: [u8; 16],
    pub(crate) memory: [u8; 4096],

    pub(crate) i: usize,
    pub(crate) pc: u16,
    pub(crate) delay_timer: u8,
    pub(crate) sound_timer: u8,

    pub(crate) stack: [u16; STACK_SIZE],
    pub(crate) sp: usize,

    /// emulator internals
    pub(crate) sdl_context: sdl2::Sdl,
    pub(crate) screen: sdl2::render::Canvas<sdl2::video::Window>,
    pub(crate) audio: Buzzer,
    pub(crate) display: [u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
    pub(crate) keyboard: [bool; KEYBOARD_SIZE],
    pub(crate) quit: bool,
    pub(crate) scale_factor: u32,
    pub(crate) display_redraw: bool,
}

pub fn initialize() -> Result<Cpu, String> {

    let scale_factor: u32 = 10;

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let title = format!("{}", env!("CARGO_PKG_NAME"));
    let window = video_subsys.window(&title, SCREEN_WIDTH * scale_factor, SCREEN_HEIGHT * scale_factor)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    
    Ok(Cpu {
        v: [0; 16],
        memory: [0; MEMORY_SIZE],
        i: 0,
        pc: 0x200,
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
        screen: window.into_canvas().build().map_err(|e| e.to_string())?,
        audio: super::sound::new(),
        display: [0u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
        keyboard: [false; KEYBOARD_SIZE],
        sdl_context: sdl_context,
        quit: false,
        scale_factor: scale_factor,
        display_redraw: true,
    })
}

impl Cpu {

    pub fn bootup(&mut self, program_buffer: Vec<u8>) {
        // load fontset
        let font_set = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,       // 0
            0x20, 0x60, 0x20, 0x20, 0x70,       // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0,       // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0,       // 3
            0x90, 0x90, 0xF0, 0x10, 0x10,       // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0,       // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0,       // 6
            0xF0, 0x10, 0x20, 0x40, 0x40,       // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0,       // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0,       // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90,       // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0,       // B
            0xF0, 0x80, 0x80, 0x80, 0xF0,       // C
            0xE0, 0x90, 0x90, 0x90, 0xE0,       // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0,       // E
            0xF0, 0x80, 0xF0, 0x80, 0x80        // F
        ];
        for i in 0..font_set.len() {
            self.memory[i] = font_set[i];
        }
        // load program
        for i in 0..program_buffer.len() {
            self.memory[i + 0x200] = program_buffer[i];
        }
        self.clear_screen();
    }


    pub fn run(&mut self) {
        loop {
            trace!("main loop");
            self.load_keyboard_status();
            if self.quit {
                self.dump();
                info!("quitting");
                break;
            }

            match self.optcode() {
                OptCode::SYS(opt) => self.sys(opt),
                OptCode::CLS(_) => self.cls(),
                OptCode::RET(_) => self.ret(),
                OptCode::LDVxByte(opt) => self.ld_vx_byte(opt),
                OptCode::SNEVxByte(opt) => self.sne_vx_byte(opt),
                OptCode::CALL(opt) => self.call(opt),
                OptCode::LDIAddr(opt) => self.ld_i_addr(opt),
                OptCode::LDIVx(opt) => self.ld_i_vx(opt),
                OptCode::JP(opt) => self.jp(opt),
                OptCode::ADDIVx(opt) => self.add_i_vx(opt),
                OptCode::LDVxI(opt) => self.ld_vx_i(opt),
                OptCode::LDFVx(opt) => self.ld_f_vx(opt),
                OptCode::DRWNibble(opt) => self.drw_vx_vy_nibble(opt),
                OptCode::ADDVxByte(opt) => self.add_vx_byte(opt),
                OptCode::ANDVxVy(opt) => self.and_vx_vy(opt),
                OptCode::SEVxByte(opt)=> self.se_vx_byte(opt),
                OptCode::SNEVxVy(opt) => self.sne_vx_vy(opt),
                OptCode::ADDVxVy(opt)=> self.add_vx_vy(opt),
                OptCode::LDVxVy(opt) => self.ld_vx_vy(opt),
                OptCode::RNDVxByte(opt) => self.rnd_vx_byte(opt),
                OptCode::SKPVx(opt) => self.skp_vx(opt),
                OptCode::SKNPVx(opt) => self.sknp_vx(opt),
                OptCode::XORVxVy(opt) => self.xor_vx_vy(opt),
                OptCode::ORVxVy(opt) => self.or_vx_vy(opt),
                OptCode::LDVxDT(opt) => self.ld_vx_dt(opt),
                OptCode::SHRVxVy(opt) => self.shr_vx_vy(opt),
                OptCode::SUBVxVy(opt) => self.sub_vx_vy(opt),
                OptCode::LDSTVx(opt) => self.ld_st_vx(opt),
                OptCode::LDDTVx(opt) => self.ld_dt_vx(opt),
                OptCode::SHLVxVy(opt) => self.shl_vx_vy(opt),
                OptCode::SEVxVy(opt) => self.se_vx_vy(opt),
                OptCode::LDBVx(opt) => self.ld_b_vx(opt),
                OptCode::LDVxK(opt)=> self.ld_vx_k(opt),


                OptCode::None(opt) => self.none(opt),
                _ => {},
            }
            self.draw();
            self.timer_tick();
            self.wait();
        }
    }

    fn timer_tick(&mut self) {
        trace!("timer ticking");
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        
        if self.sound_timer > 0 {
            self.audio.start();
        } else {
            self.audio.stop();
        }
    }

    pub(crate) fn wait(&mut self) {
        thread::sleep(WAIT_INTERVAL);        
    }

    fn optcode(&mut self) -> OptCode {
        let (a, b) = (self.memory[self.pc as usize], self.memory[(self.pc + 1) as usize]);
        let opt = u16::from_be_bytes([a, b]);
        debug!("optcode => {:#X} {:#X}", opt, (opt & 0xF000));
        match opt & 0xF000 {
            0x0000 =>  match opt & 0x00FF {
                0x00E0 => OptCode::CLS(opt),
                0x00EE => OptCode::RET(opt),
                _ => OptCode::SYS(opt),
            },
            0x1000 => OptCode::JP(opt),
            0x2000 => OptCode::CALL(opt),
            0x3000 => OptCode::SEVxByte(opt),
            0x4000 => OptCode::SNEVxByte(opt),
            0x5000 => OptCode::SEVxVy(opt),
            0x6000 => OptCode::LDVxByte(opt),
            0x7000 => OptCode::ADDVxByte(opt),
            0x8000 => match opt & 0x000F {
                0x0000 => OptCode::LDVxVy(opt),
                0x0001 => OptCode::ORVxVy(opt),
                0x0002 => OptCode::ANDVxVy(opt),
                0x0003 => OptCode::XORVxVy(opt),
                0x0004 => OptCode::ADDVxVy(opt),
                0x0005 => OptCode::SUBVxVy(opt),
                0x0006 => OptCode::SHRVxVy(opt),
                0x000E => OptCode::SHLVxVy(opt),
                _ => OptCode::None(opt),
            },
            0x9000 => OptCode::SNEVxVy(opt),
            0xA000 => OptCode::LDIAddr(opt),
            0xC000 => OptCode::RNDVxByte(opt),
            0xD000 => OptCode::DRWNibble(opt),
            0xE000 => match opt & 0x00FF {
                0x009E => OptCode::SKPVx(opt),
                0x00A1 => OptCode::SKNPVx(opt),
                _ => OptCode::None(opt),
            }, 
            0xF000 => match opt & 0x00FF {
                0x0007 => OptCode::LDVxDT(opt),
                0x000A => OptCode::LDVxK(opt),
                0x0015 => OptCode::LDDTVx(opt),
                0x0018 => OptCode::LDSTVx(opt),
                0x001E => OptCode::ADDIVx(opt),
                0x0029 => OptCode::LDFVx(opt),
                0x0033 => OptCode::LDBVx(opt),
                0x0055 => OptCode::LDIVx(opt),
                0x0065 => OptCode::LDVxI(opt),
                _ => OptCode::None(opt),
            },

            _ => OptCode::None(opt),
        }
    }

    pub(crate) fn dump(&mut self) {
        println!("=======   DUMPING     =======");

        println!("i = {}", self.i);
        println!("pc = {}", self.pc);
        println!("dt = {}", self.delay_timer);
        println!("st = {}", self.sound_timer);
        println!("sp = {}", self.sp);
        
        println!("======   STACK       =======");
        for a in 0..self.stack.len() {
            println!("s[{:#X}] = {:#X}", a, self.stack[a]);
        }        

        println!("======   REGISTER    =======");
        for a in 0..self.v.len() {
            println!("v[{:#X}] = {:#X}", a, self.v[a]);
        }

        println!("======   DISPLAY     =======");
        for y in 0..SCREEN_HEIGHT as usize {                
            for x in 0..SCREEN_WIDTH as usize {
                print!("{}", self.display[x + y * SCREEN_WIDTH as usize]);
            }
            println!("");
        }
    }
}
