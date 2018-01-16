extern crate sfml;
extern crate time;

use chip8;
use display::Display;

const WINDOW_W: usize = 800;
const WINDOW_H: usize = 400;
const WINDOW_LEN: usize = WINDOW_W * WINDOW_H; 

pub struct DisplaySFML {
      window: sfml::graphics::RenderWindow,
      texture_data: [u8; chip8::DISPLAY_SIZE * 4],
      texture: sfml::graphics::Texture,
}

impl DisplaySFML {
      pub fn new() -> DisplaySFML {
            use self::sfml::window::{Event, Style};
            use self::sfml::graphics::RenderWindow;

            let mut window = RenderWindow::new(
                  (WINDOW_W as u32, WINDOW_H as u32),
                  "CHIP-8",
                  Style::CLOSE,
                  &Default::default()
            );
            window.set_framerate_limit(60);

            let mut disp = DisplaySFML {
                  window,
                  texture_data: [0; chip8::DISPLAY_SIZE * 4],
                  texture: sfml::graphics::Texture::new(chip8::DISPLAY_W as u32, chip8::DISPLAY_H as u32).unwrap()
            };
            disp.texture.set_repeated(false);
            return disp;
      }
}

impl Display for DisplaySFML {
      fn update(&mut self) {
            use self::sfml::window::Event;
            while let Some(ev) = self.window.poll_event() {
                  if ev == Event::Closed {
                        self.window.close();
                  }
            }
      }
      fn draw(&mut self, video: &[bool; chip8::DISPLAY_SIZE]) {
            use self::sfml::graphics::{RenderTarget, Transformable};

            let bstart = time::PreciseTime::now();

            for i in 0..chip8::DISPLAY_SIZE {
                  self.texture_data[i*4] = if video[i] { 255 } else { 0 };
                  self.texture_data[i*4+1] = if video[i] { 255 } else { 0 };
                  self.texture_data[i*4+2] = if video[i] { 255 } else { 0 };
                  self.texture_data[i*4+3] = 255;
            }
            self.texture.update_from_pixels(&self.texture_data, chip8::DISPLAY_W as u32, chip8::DISPLAY_H as u32, 0, 0);
            let mut sprite = sfml::graphics::Sprite::with_texture(&self.texture);
            sprite.set_scale(sfml::system::Vector2f::new(12.5, 12.5));
            self.window.draw(&sprite);
            self.window.display();

            let bend = time::PreciseTime::now();
            println!("{} millis", bstart.to(bend).num_milliseconds());
      }
      fn should_close(&self) -> bool {
            !self.window.is_open()
      }
}