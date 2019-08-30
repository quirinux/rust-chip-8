//use sdl2::event::Event;
//use sdl2::pixels;
//use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;


use super::cpu::*;

impl Cpu {

    pub(crate) fn clear_screen(&mut self) {
        //self.screen.set_draw_color(pixels::Color::RGB(0xFF, 0xFF, 0xFF));
        self.display = [0u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize];
    }
    
    pub(crate) fn draw(&mut self) {
        if !self.display_redraw {
            return;
        }
        let texture_creator = self.screen.texture_creator();
        let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string()).unwrap();
        
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            trace!("pitch:{} height:{} width:{}", pitch, SCREEN_HEIGHT, SCREEN_WIDTH);
            for y in 0..SCREEN_HEIGHT as usize {
                for x in 0..SCREEN_WIDTH as usize {
                    let offset = y*pitch + x*3;
                    trace!("y*pitch + x*3 => {}*{} + {}*3 = offset:{}", y, pitch, x, offset);
                    trace!("x:{} y:{} screen_width:{}", x, y, SCREEN_WIDTH);
                    trace!("x + y * self.screen_width = {}", x + y * SCREEN_WIDTH as usize);
                    let state = if self.display[x + y * SCREEN_WIDTH as usize] == 1 {255} else {0};
                    buffer[offset] = state;
                    buffer[offset + 1] = state;
                    buffer[offset + 2] = state;
                    //print!("{}", state);
                }
                //println!("");
            }
        }).unwrap();

        self.screen.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH * self.scale_factor, SCREEN_HEIGHT * self.scale_factor))).unwrap();
        self.screen.present();
        self.display_redraw = false;
    }
}

