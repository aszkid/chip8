extern crate sfml;
extern crate time;

use chip8;
use display::Display;

const WINDOW_W: usize = 1200;
const WINDOW_H: usize = 600;
const WINDOW_LEN: usize = WINDOW_W * WINDOW_H; 

use self::sfml::window::Key;
use self::sfml::system::Vector2f;


const KEY_BINDINGS: [(u8, Key); 16] = [
      (0x1, Key::Num1),
      (0x2, Key::Num2),
      (0x3, Key::Num3),
      (0xC, Key::Num4),
      (0x4, Key::Q),
      (0x5, Key::W),
      (0x6, Key::E),
      (0xD, Key::R),
      (0x7, Key::A),
      (0x8, Key::S),
      (0x9, Key::D),
      (0xE, Key::F),
      (0xA, Key::Z),
      (0x0, Key::X),
      (0xB, Key::C),
      (0xF, Key::V)
];

fn key_local_to_chip(k: Key) -> Option<u8> {
      for pair in KEY_BINDINGS.iter() {
            if pair.1 == k {
                  return Some(pair.0);
            }
      }
      None
}

pub struct DisplaySFML {
      window: sfml::graphics::RenderWindow,
      texture_data: [u8; chip8::DISPLAY_SIZE * 4],
      texture: sfml::graphics::Texture,
      font: sfml::graphics::Font
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

            let mut disp = DisplaySFML {
                  window,
                  texture_data: [0; chip8::DISPLAY_SIZE * 4],
                  texture: sfml::graphics::Texture::new(chip8::DISPLAY_W as u32, chip8::DISPLAY_H as u32).unwrap(),
                  font: sfml::graphics::Font::from_file("res/Hack-Regular.ttf").unwrap()
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
                              if let Some(key) = key_local_to_chip(code) {
                                    chip.key_pressed = key;
                              }
                        },
                        _ => ()
                  }
            }

            // Update keypad state
            for pair in KEY_BINDINGS.iter() {
                  chip.keypad[pair.0 as usize] = pair.1.is_pressed();
            }
      }
      fn draw(&mut self, chip: &chip8::Chip) {
            use self::sfml::graphics::{RenderTarget, Transformable};

            for i in 0..chip8::DISPLAY_SIZE {
                  let mut color = [22, 34, 56, 255];
                  if chip.display[i]  {
                        color = [116, 163, 252, 255];
                  }
                  self.texture_data[i*4] = color[0];
                  self.texture_data[i*4+1] = color[1];
                  self.texture_data[i*4+2] = color[2];
                  self.texture_data[i*4+3] = color[3];
            }
            self.texture.update_from_pixels(&self.texture_data, chip8::DISPLAY_W as u32, chip8::DISPLAY_H as u32, 0, 0);
            let mut sprite = sfml::graphics::Sprite::with_texture(&self.texture);
            //sprite.set_scale(sfml::system::Vector2f::new(WINDOW_W as f32 / chip8::DISPLAY_W as f32, WINDOW_H as f32 / chip8::DISPLAY_H as f32));
            sprite.set_scale(Vector2f::new(12.5, 12.5));

            // debugging
            let string = String::from("Hello world");
            let mut text = sfml::graphics::Text::new(&string, &self.font, 12);
            text.set_fill_color(&sfml::graphics::Color::WHITE);
            text.set_position(Vector2f::new(805.0, 0.0));

            self.window.draw(&text);
            self.window.draw(&sprite);
            self.window.display();
      }
      fn should_close(&self) -> bool {
            !self.window.is_open()
      }
}