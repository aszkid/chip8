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
      fn update(&mut self, chip: &mut chip8::Chip) {
            use self::sfml::window::{Event, Key};
            while let Some(ev) = self.window.poll_event() {
                  match ev {
                        Event::Closed => self.window.close(),
                        Event::KeyPressed { code, .. } => {
                              chip.key_pressed = match code {
                                    Key::Numpad0 => 0x0,
                                    Key::Numpad1 => 0x1,
                                    Key::Numpad2 => 0x2,
                                    Key::Numpad3 => 0x3,
                                    Key::Numpad4 => 0x4,
                                    Key::Numpad5 => 0x5,
                                    Key::Numpad6 => 0x6,
                                    Key::Numpad7 => 0x7,
                                    Key::Numpad8 => 0x8,
                                    Key::Numpad9 => 0x9,
                                    Key::A => 0xA,
                                    Key::B => 0xB,
                                    Key::C => 0xC,
                                    Key::D => 0xD,
                                    Key::E => 0xE,
                                    Key::F => 0xF,
                                    _ => 0x10
                              };
                        },
                        _ => ()
                  }
            }
      }
      fn draw(&mut self, chip: &chip8::Chip) {
            use self::sfml::graphics::{RenderTarget, Transformable};

            let bstart = time::PreciseTime::now();

            for i in 0..chip8::DISPLAY_SIZE {
                  self.texture_data[i*4] = if chip.display[i] { 255 } else { 0 };
                  self.texture_data[i*4+1] = if chip.display[i] { 255 } else { 0 };
                  self.texture_data[i*4+2] = if chip.display[i] { 255 } else { 0 };
                  self.texture_data[i*4+3] = 255;
            }
            self.texture.update_from_pixels(&self.texture_data, chip8::DISPLAY_W as u32, chip8::DISPLAY_H as u32, 0, 0);
            let mut sprite = sfml::graphics::Sprite::with_texture(&self.texture);
            sprite.set_scale(sfml::system::Vector2f::new(12.5, 12.5));
            self.window.draw(&sprite);
            self.window.display();

            let bend = time::PreciseTime::now();
            //println!("{} millis", bstart.to(bend).num_milliseconds());
      }
      fn should_close(&self) -> bool {
            !self.window.is_open()
      }
}