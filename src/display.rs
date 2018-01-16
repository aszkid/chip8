use chip8;

pub trait Display {
      fn draw(&mut self, chip: &chip8::Chip);
      fn update(&mut self, chip: &mut chip8::Chip);
      fn should_close(&self) -> bool;
}