extern crate sfml;

use chip8;
use display::Display;

const WINDOW_W: usize = 800;
const WINDOW_H: usize = 400;
const WINDOW_LEN: usize = WINDOW_W * WINDOW_H; 

pub struct DisplaySFML {
      window: sfml::graphics::RenderWindow,
      texture_data: [u8; WINDOW_LEN * 4],
      texture: sfml::graphics::Texture,
}

impl DisplaySFML {
      pub fn new() -> DisplaySFML {
            use self::sfml::window::{Event, Style};
            use self::sfml::graphics::RenderWindow;
            let mut window = RenderWindow::new(
                  (800, 400),
                  "CHIP-8",
                  Style::CLOSE,
                  &Default::default()
            );
            window.set_framerate_limit(60);

            DisplaySFML {
                  window,
                  texture_data: [0; WINDOW_LEN * 4],
                  texture: sfml::graphics::Texture::new(WINDOW_W as u32, WINDOW_H as u32).unwrap() 
            }
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
      fn draw(&mut self, video: &[u8; chip8::DISPLAY_SIZE]) {
            use self::sfml::graphics::RenderTarget;
            for i in 0..WINDOW_LEN {
                  self.texture_data[i] = ((i as f32)/(WINDOW_LEN as f32) * 255.0) as u8;
                  self.texture_data[i+1] = (((i % WINDOW_H) as f32)/(WINDOW_LEN as f32) * 255.0) as u8;
                  self.texture_data[i+2] = (((i % WINDOW_W) as f32)/(WINDOW_LEN as f32) * 255.0) as u8;
                  self.texture_data[i+3] = 255;
            }
            self.texture.update_from_pixels(&self.texture_data, WINDOW_W as u32, WINDOW_H as u32, 0, 0);
            let mut sprite = sfml::graphics::Sprite::with_texture(&self.texture);
            self.window.draw(&sprite);
            self.window.display();
      }
      fn should_close(&self) -> bool {
            !self.window.is_open()
      }
}