use chip8;

pub trait Display {
      fn draw(&mut self, video: &[u8; chip8::DISPLAY_SIZE]);
      fn update(&mut self);
      fn should_close(&self) -> bool;
}