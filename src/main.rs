extern crate time;

mod chip8;
mod display;
mod display_sfml;

use display::Display;

fn main() {

      const CPU_FREQUENCY: f32 = 500.0; // Hz
      const DRAW_FREQUENCY: f32 = 60.0; // Hz

      let rom = match std::env::args().nth(1) {
            Some(v) => v,
            None => panic!("Need ROM to load!")
      };
      println!("Playing ROM `{}`", rom);

      let mut display = display_sfml::DisplaySFML::new();
      let mut chip = chip8::Chip::new();
      chip.load_rom(&format!("roms/{}", &rom));

      let mut begin_cpu = time::PreciseTime::now();
      let mut begin_display = begin_cpu.clone();
      'running: loop {
            let now = time::PreciseTime::now();

            let delta_cpu = begin_cpu.to(now);
            if delta_cpu.num_milliseconds() >= (1000.0 / CPU_FREQUENCY).round() as i64 {
                  begin_cpu = now.clone();

                  if chip.running {
                        chip.cycle();
                  }
            }

            let delta_display = begin_display.to(now);
            if delta_display.num_milliseconds() >= (1000.0 / DRAW_FREQUENCY).round() as i64 {
                  begin_display = now.clone();

                  display.update(&mut chip);
                  if display.should_close() {
                        break 'running;
                  }
                  display.draw(&chip);
            }
      }

      chip.dump();
}