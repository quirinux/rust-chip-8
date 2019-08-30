use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use super::cpu::*;


impl Cpu {

    pub(crate) fn clear_keyboard(&mut self) {
        //self.sdl_context.event_pump().unwrap();
        self.keyboard = [false; KEYBOARD_SIZE];
    }
    
    pub(crate) fn load_keyboard_status(&mut self) {
        trace!("loading keyboard status");
        self.clear_keyboard();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Num0), ..} => self.keyboard[0x0] = true,
                Event::KeyDown {keycode: Some(Keycode::Num1), ..} => self.keyboard[0x1] = true,
                Event::KeyDown {keycode: Some(Keycode::Num2), ..} => self.keyboard[0x2] = true,
                Event::KeyDown {keycode: Some(Keycode::Num3), ..} => self.keyboard[0x3] = true,
                Event::KeyDown {keycode: Some(Keycode::Num4), ..} => self.keyboard[0x4] = true,
                Event::KeyDown {keycode: Some(Keycode::Num5), ..} => self.keyboard[0x5] = true,
                Event::KeyDown {keycode: Some(Keycode::Num6), ..} => self.keyboard[0x6] = true,
                Event::KeyDown {keycode: Some(Keycode::Num7), ..} => self.keyboard[0x7] = true,
                Event::KeyDown {keycode: Some(Keycode::Num8), ..} => self.keyboard[0x8] = true,
                Event::KeyDown {keycode: Some(Keycode::Num9), ..} => self.keyboard[0x9] = true,
                Event::KeyDown {keycode: Some(Keycode::A), ..} => self.keyboard[0xA] = true,
                Event::KeyDown {keycode: Some(Keycode::B), ..} => self.keyboard[0xB] = true,
                Event::KeyDown {keycode: Some(Keycode::C), ..} => self.keyboard[0xC] = true,
                Event::KeyDown {keycode: Some(Keycode::D), ..} => self.keyboard[0xD] = true,
                Event::KeyDown {keycode: Some(Keycode::E), ..} => self.keyboard[0xE] = true,
                Event::KeyDown {keycode: Some(Keycode::F), ..} => self.keyboard[0xF] = true,

                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Q), ..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => self.quit = true,
                Event::KeyDown {keycode: Some(Keycode::Z), ..} => self.dump(),


                _ => {},
            }
        }
    }
}
    
