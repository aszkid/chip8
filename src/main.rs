mod chip8;
mod display;
mod display_sfml;

use display::Display;

fn main() {

      // Create gl display
      let mut display = display_sfml::DisplaySFML::new();
      // Create chip instance
      let mut chip = chip8::Chip::new();
      chip.load_rom("roms/draw.rom");

      'running: loop {
            
            if chip.running {
                  chip.cycle();
            }

            display.update();
            display.draw(&chip.display);
            if display.should_close() {
                  break 'running;
            }
      }

      chip.dump();
}