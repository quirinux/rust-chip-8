extern crate rand;
use rand::Rng;

use std::num::Wrapping;

use super::cpu::*;

macro_rules! add {
    ($a:expr, $b:expr) => {{ (Wrapping($a) + Wrapping($b)).0 }}
}

macro_rules! sub {
    ($a:expr, $b:expr) => {{ (Wrapping($a) - Wrapping($b)).0 }}
}

macro_rules! times {
    ($a:expr, $b:expr) => {{ (Wrapping($a) * Wrapping($b)).0 }}
}

macro_rules! by {
    ($a:expr, $b:expr) => {{ (Wrapping($a) / Wrapping($b)).0 }}
}

macro_rules! nnn {
    ($optcode:expr) => {{ ($optcode & 0x0FFF) }}
}

macro_rules! vx_byte {
    ($optcode:expr) => {{ 
        let _be = $optcode.to_be_bytes();
        let _vx: usize = (_be[0] & 0x0F).into();
        let _byte: u8 = _be[1];
        (_vx, _byte)
    }}
}

macro_rules! vx_vy {
    ($optcode:expr) => {{
        let _be = $optcode.to_be_bytes();
        let _vx: u8 = (_be[0] & 0x0F).into();
        let _vy: u8 = (_be[1] & 0xF0) >> 4;
        (_vx as usize, _vy as usize)            
    }}
}

macro_rules! tern {
    ($term:expr, $true:expr, $false:expr) => {{
        if $term {
            $true
        } else {
            $false
        }
    }}
}

macro_rules! skip_if {
    ($term:expr) => {{
        tern!($term, 4, 2)
    }}
}

#[derive(Debug)]
pub(crate) enum OptCode {
    SYS(u16),
    CLS(u16),
    RET(u16),
    JP(u16),
    CALL(u16),
    SEVxByte(u16),
    SEVxVy(u16),
    SNEVxByte(u16),
    LDVxByte(u16),
    ADDVxByte(u16),
    LDVxVy(u16),
    ORVxVy(u16),
    ANDVxVy(u16),
    XORVxVy(u16),
    ADDVxVy(u16),
    SUBVxVy(u16),
    SHRVxVy(u16),
    SUBNVxVy(u16),
    SHLVxVy(u16),
    SNEVxVy(u16),
    LDIAddr(u16),
    JPV0Addr(u16),
    RNDVxByte(u16),
    DRWNibble(u16),
    SKPVx(u16),
    SKNPVx(u16),
    LDVxDT(u16),
    LDVxK(u16),
    LDDTVx(u16),
    LDSTVx(u16),
    ADDIVx(u16),
    LDFVx(u16),
    LDBVx(u16),
    LDIVx(u16),
    LDVxI(u16),

    // suoer chip-8 instructions
    SCR(u16),
    SCL(u16),
    EXIT(u16),
    LOW(u16),
    HIGH(u16),
    DRWVxVy(u16),
    LDHFVx(u16),
    LDRVx(u16),
    LVVxR(u16),
    
    // in case of instruction not implemented
    None(u16),
}

/// Implementation of the opcodes for cpu struct
/// ref: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
impl Cpu {
    /// Intercepts all non defined optcode
    /// there is no relation to chip-8 specification
    /// it's internal to this project only
    pub(crate) fn none(&mut self, optcode: u16) {
        error!("None => {:#X}, v:{:?}", optcode, self.v);
        self.quit = true;
    }

    /// 0nnn - SYS addr
    /// Jump to a machine code routine at nnn.
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    pub(crate) fn sys(&mut self, optcode: u16) {
        warn!("SYS => {:#X}", optcode);
        self.pc += 2;
    }
    
    /// 00E0 - CLS
    /// Clear the display.
    pub(crate) fn cls(&mut self){
        debug!("CLS - done");
        self.clear_screen();
        self.display_redraw = true;
        self.pc += 2;
    }

    /// 00EE - RET
    /// Return from a subroutine.
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    pub(crate) fn ret(&mut self) {
        debug!("RET - done");
        self.sp_decrement();
        self.pc = self.stack[self.sp]; 
        self.pc += 2;
        //self.quit = true;
    }
    
    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    pub(crate) fn call(&mut self, optcode: u16) {
        debug!("CALL => {:#X} - done", optcode);
        self.stack[self.sp] = self.pc;
        self.sp_increment();
        self.pc = nnn!(optcode);
        //self.quit = true;
    }
    
    /// 6xkk - LD Vx, byte
    /// Set Vx = kk.
    /// The interpreter puts the value kk into register Vx.
    pub(crate) fn ld_vx_byte(&mut self, optcode: u16) {
        debug!("LDVxByte => {:#X} - done", optcode);
        let (_vx, _byte) = vx_byte!(optcode);
        self.v[_vx] = _byte;
        self.pc += 2;
        //self.quit = true;
    }

    /// 4xkk - SNE Vx, byte
    /// Skip next instruction if Vx != kk.
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    pub(crate) fn sne_vx_byte(&mut self, optcode: u16) {
        debug!("SNEVxByte => {:#X} - done", optcode);
        let (_vx, _byte) = vx_byte!(optcode);
        self.pc += skip_if!(self.v[_vx] != _byte);
        //self.quit = true;
    }

    /// Annn - LD I, addr
    /// Set I = nnn.
    /// The value of register I is set to nnn.
    pub(crate) fn ld_i_addr(&mut self, optcode: u16) {
        debug!("LDIAddr => {:#X} - done", optcode);
        self.i = nnn!(optcode).into();
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    pub(crate) fn ld_i_vx(&mut self, optcode: u16) {
        debug!("LDIVx => {:#X} - done", optcode);
        let (_vx, _) = vx_vy!(optcode);
        for x in 0..=_vx {
            trace!("memory[{}] = {}", self.i + x, self.v[x]);
            self.memory[self.i + x] = self.v[x];
        }
        self.pc += 2;
        //self.quit = true;
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    pub(crate) fn jp(&mut self, optcode: u16) {
        debug!("JP => {:#X} - done", optcode);
        self.pc = nnn!(optcode);
        //self.quit = true;
    }

    ///  Fx1E - ADD I, Vx
    /// Set I = I + Vx.
    /// The values of I and Vx are added, and the results are stored in I.
    pub(crate) fn add_i_vx(&mut self, optcode: u16) {
        debug!("ADDIVx => {:#X} - done", optcode);
        let (_x, _) = vx_vy!(optcode);
        self.i += self.v[_x] as usize;
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I.
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    pub(crate) fn ld_vx_i(&mut self, optcode: u16) {
        debug!("LDVxI => {:#X} - done", optcode);
        let (_vx, _) = vx_vy!(optcode);
        for x in 0..=_vx {
            self.v[x] = self.memory[self.i + x];
        }
        self.pc += 2;
        //self.quit = true;
    }
    /// Fx29 - LD F, Vx
    /// Set I = location of sprite for digit Vx.
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    pub(crate) fn ld_f_vx(&mut self, optcode: u16) {
        debug!("LDFVx => {:#X} - done", optcode);
        let (_vx, _) = vx_vy!(optcode);
        self.i = (self.v[_vx] * 5).into();
        self.pc += 2;
        //self.quit = true;
    }

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    /// Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so
    /// part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR,
    /// and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    pub(crate) fn drw_vx_vy_nibble(&mut self, optcode: u16) {
        debug!("DRWNibble => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        let _vx = self.v[_x];
        let _vy = self.v[_y];
        let n = optcode & 0x000F;
        self.v[0xF] &= 0;
        for y in 0..n as u8 {
            let pixel = self.memory[self.i + y as usize];
            for x in 0..8 as u8 {
                if pixel & (0x80 >> x) >= 1 {
                    let address =
                        (add!(x, _vx) as u32 % SCREEN_WIDTH) as usize +
                        (add!(y, _vy) as u32 % SCREEN_HEIGHT) as usize *
                        SCREEN_WIDTH as usize;
                    
                    if address >= self.display.len() {
                        error!("address({}) hits display outbounds({})", address, self.display.len() - 1);
                        error!("x:{} _x:{} x+_x:{}", x, _vx, add!(x, _vx));
                        error!("y:{} _y:{} y+_y:{}", y, _vy, add!(y, _vy));
                        error!("x+y*screen_width:{}", add!(x, _vx) as usize + add!(y, _vy) as usize * SCREEN_WIDTH as usize);                        
                        self.quit = true;
                        return;
                    }
                    self.v[0xF] |= self.display[address];
                    self.display[address] ^= 1;
                }
            }
        }
        self.display_redraw = true;
        self.pc += 2;
    }

    /// 7xkk - ADD Vx, byte
    /// Set Vx = Vx + kk.
    /// Adds the value kk to the value of register Vx, then stores the result in Vx. 
    pub(crate) fn add_vx_byte(&mut self, optcode: u16) {
        debug!("ADDVxByte => {:#X} - done", optcode);
        let (_x, _byte) = vx_byte!(optcode);
        self.v[_x] = add!(self.v[_x], _byte);
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy.
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    /// A bitwise AND compares the corrseponding bits from two values, and if both bits are 1,
    /// then the same bit in the result is also 1. Otherwise, it is 0. 
    pub(crate) fn and_vx_vy(&mut self, optcode: u16) {
        debug!("ANDVxVy => {:#X} - done", optcode);
        let (_vx, _vy) = vx_vy!(optcode);
        self.v[_vx] &= self.v[_vy];
        self.pc += 2;
        //self.quit = true;
    }

    /// 3xkk - SE Vx, byte
    /// Skip next instruction if Vx = kk.
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    pub(crate) fn se_vx_byte(&mut self, optcode: u16) {
        debug!("SEVxByte => {:#X} - done", optcode);
        let (_vx, _byte) = vx_byte!(optcode);
        self.pc += skip_if!(self.v[_vx] == _byte);
        //self.quit = true;
    }

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy.
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    pub(crate) fn sne_vx_vy(&mut self, optcode: u16) {
        debug!("SNEVxVy => {:#X} - done", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.pc += skip_if!(self.v[_x] != self.v[_y]);
        //self.quit = true;
    }
    
    /// 8xy4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry.
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    pub(crate) fn add_vx_vy(&mut self, optcode: u16) {
        debug!("ADDVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        let _vx: usize = self.v[_x] as usize;
        let _vy: usize = self.v[_y] as usize;
        let result = add!(_vx, _vy);
        self.v[_x] = result as u8;
        self.v[0xF] = tern!(result > 255, 1, 0);
        if result > 255 {
            trace!("ADDVxVy => result({})", result);
        }
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    /// Stores the value of register Vy in register Vx.
    pub(crate) fn ld_vx_vy(&mut self, optcode: u16) {
        debug!("LDVxVy => {:#X}", optcode);
        let (_vx, _vy) = vx_vy!(optcode);
        self.v[_vx] = self.v[_vy];
        self.pc += 2;
        //self.quit = true;
    }

    /// Cxkk - RND Vx, byte
    /// Set Vx = random byte AND kk.
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    /// The results are stored in Vx. See instruction 8xy2 for more information on AND.
    pub(crate) fn rnd_vx_byte(&mut self, optcode: u16) {
        debug!("RNDVxByte => {:#X} - done", optcode);
        let rnd: u8 = rand::thread_rng().gen_range(0, 255).into();
        let (_vx, _byte) = vx_byte!(optcode);
        self.v[_vx] = rnd & _byte;
        self.pc += 2;
        //self.quit = true;
    }

    /// Ex9E - SKP Vx
    /// Skip next instruction if key with the value of Vx is pressed.
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    pub(crate) fn skp_vx(&mut self, optcode: u16) {
        debug!("SKPVx => {:#X}", optcode);
        let (_x, _ ) = vx_vy!(optcode);
        let _vx = self.v[_x] as usize;
        info!("k[{}]: {}", _vx, self.keyboard[_vx]);
        self.pc += skip_if!(self.keyboard[_vx]);
        //self.quit = true;
    }

    /// ExA1 - SKNP Vx
    /// Skip next instruction if key with the value of Vx is not pressed.
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    pub(crate) fn sknp_vx(&mut self, optcode: u16) {
        debug!("SKNPVx => {:#X}", optcode);
        let (_x, _ ) = vx_vy!(optcode);
        let _vx = self.v[_x] as usize;
        info!("k[{}]: {}", _vx, self.keyboard[_vx]);
        self.pc += skip_if!(!self.keyboard[_vx]);
        //self.quit = true;
    }

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy.
    /// Performs a bitwise exclusive OR on the values of Vx and Vy,
    /// then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values,
    /// and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    pub(crate) fn xor_vx_vy(&mut self, optcode: u16) {
        debug!("XORVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.v[_x] ^= self.v[_y];
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy.
    /// Performs a bitwise OR on the values of Vx and Vy,
    /// then stores the result in Vx. A bitwise OR compares
    /// the corrseponding bits from two values, and if either bit is 1,
    /// then the same bit in the result is also 1. Otherwise, it is 0.
    pub(crate) fn or_vx_vy(&mut self, optcode: u16) {
        debug!("ORVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.v[_x] |= self.v[_y];
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx07 - LD Vx, DT
    /// Set Vx = delay timer value.
    /// The value of DT is placed into Vx.
    pub(crate) fn ld_vx_dt(&mut self, optcode: u16) {
        debug!("LDVxDT => {:#X} - done", optcode);
        let (_vx, _) = vx_vy!(optcode);
        self.v[_vx] = self.delay_timer;
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx SHR 1.
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    pub(crate) fn shr_vx_vy(&mut self, optcode: u16) {
        debug!("SHRVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.v[_x] = by!(self.v[_x], 2);
        self.v[0xF] = tern!(self.v[_x] & (1 << 7) == 0, 0, 1);
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xy5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    /// If Vx > Vy, then VF is set to 1, otherwise 0.
    /// Then Vy is subtracted from Vx, and the results stored in Vx.
    pub(crate) fn sub_vx_vy(&mut self, optcode: u16) {
        debug!("SUBVxVy => {:#X} - done", optcode);
        let (_vx, _vy) = vx_vy!(optcode);
        self.v[0xF] = tern!(self.v[_vx] > self.v[_vy], 1, 0);
        self.v[_vx] = sub!(self.v[_vx], self.v[_vy]);
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx18 - LD ST, Vx
    /// Set sound timer = Vx.
    /// ST is set equal to the value of Vx.
    pub(crate) fn ld_st_vx(&mut self, optcode: u16) {
        debug!("LDSTVx => {:#X}", optcode);
        let (_x, _) = vx_vy!(optcode);
        self.sound_timer = self.v[_x];
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx15 - LD DT, Vx
    /// Set delay timer = Vx.
    /// DT is set equal to the value of Vx.
    pub(crate) fn ld_dt_vx(&mut self, optcode: u16) {
        debug!("LDDTVx => {:#X} - done", optcode);
        let (_vx, _) = vx_vy!(optcode);
        self.delay_timer = self.v[_vx];
        self.pc += 2;
        //self.quit = true;
    }

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx SHL 1.
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    pub(crate) fn shl_vx_vy(&mut self, optcode: u16) {
        debug!("SHLVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.v[_x] = times!(self.v[_x], 2);
        self.v[0xF] = tern!(self.v[_x] & (1 << 7) == 0, 0, 1);
        self.pc += 2;
        //self.quit = true;
    }

    /// 5xy0 - SE Vx, Vy
    /// Skip next instruction if Vx = Vy.
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    pub(crate) fn se_vx_vy(&mut self, optcode: u16) {
        debug!("SEVxVy => {:#X}", optcode);
        let (_x, _y) = vx_vy!(optcode);
        self.pc += skip_if!(self.v[_x] == self.v[_y]);
        //self.quit = true;
    }

    /// Fx33 - LD B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    pub(crate) fn ld_b_vx(&mut self, optcode: u16) {
        debug!("LDBVx => {:#X} - done", optcode);
        // let (_vx, _) = self.vx_vy(optcode);
        let (_x, _) = vx_vy!(optcode);
        let _vx = self.v[_x];
        self.memory[self.i] = _vx / 100;
        self.memory[self.i + 1] = _vx / 10 % 10;
        self.memory[self.i + 2] = _vx % 10;
        self.pc += 2;
        //self.quit = true;
    }

    /// Fx0A - LD Vx, K
    /// Wait for a key press, store the value of the key in Vx.
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    pub(crate) fn ld_vx_k(&mut self, optcode: u16) {
        debug!("LDVxK => {:#X}", optcode);
        let (_vx, _) = vx_vy!(optcode);
        for i in 0..self.keyboard.len() {
            if self.keyboard[i] {
                self.v[_vx] = i as u8;
                self.pc += 2;
            }
        }
        //self.quit = true;
    }


    /// the following functions are for internal use only
    /// they are related to flow control and are meant
    /// to be reused

    /// increment stack point index value
    fn sp_increment(&mut self) {
        if self.sp < STACK_SIZE - 1 {
            self.sp += 1;
        } else {
            self.sp = 0;
        }
    }

    /// decrement stack point index value
    fn sp_decrement(&mut self) {
        if self.sp > 0 {
            self.sp -= 1;
        } else {
            self.sp = STACK_SIZE - 1;
        }
    }
}


