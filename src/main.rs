extern crate time;

mod chip8;
mod display;
mod display_sfml;

use display::Display;

fn main() {

      // Create gl display
      let mut display = display_sfml::DisplaySFML::new();
      // Create chip instance
      let mut chip = chip8::Chip::new();
      chip.load_rom("roms/pong2.rom");

      let mut begin_cpu = time::PreciseTime::now();
      let mut begin_display = begin_cpu.clone();
      'running: loop {
            let now = time::PreciseTime::now();

            let delta_cpu = begin_cpu.to(now);
            if delta_cpu.num_milliseconds() > 2 {
                  begin_cpu = now.clone();

                  if chip.running {
                        chip.cycle();
                  }

                  display.update(&mut chip);
                  if display.should_close() {
                        break 'running;
                  }
            }

            let delta_display = begin_display.to(now);
            if delta_display.num_milliseconds() > 16 {
                  println!("Display tick!");
                  begin_display = now.clone();

                  display.draw(&chip);
            }
      }

      chip.dump();
}