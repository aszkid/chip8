extern crate time;
extern crate sfml;

mod chip8;
mod display;
mod display_sfml;

use display::Display;

const BEEP_SAMPLE_RATE: u32 = 44100;
const BEEP_FREQUENCY: f32 = 1000.0;
fn generate_beep() -> Vec<i16> {
      const SAMPLES: usize = BEEP_SAMPLE_RATE as usize / 2;
      let mut data: Vec<i16> = Vec::new();
      data.resize(SAMPLES, 0);

      // generate half a second of a sine wave
      for i in 0..SAMPLES {
            let t = i as f32 / BEEP_SAMPLE_RATE as f32;
            data[i] = ((BEEP_FREQUENCY * t * 2.0 * 3.1415).sin() * 32767.0).round() as i16;
      }

      data
}

fn main() {

      const CPU_FREQUENCY: f32 = 500.0; // Hz
      const DRAW_FREQUENCY: f32 = 60.0; // Hz

      let rom = match std::env::args().nth(1) {
            Some(v) => v,
            None => panic!("Need ROM to load!")
      };
      println!("Playing ROM `{}`", rom);

      let beep_raw = generate_beep();
      let beep_buffer = sfml::audio::SoundBuffer::from_samples(
            beep_raw.as_slice(),
            1, // channel_count
            BEEP_SAMPLE_RATE // sample_rate
      ).unwrap();
      beep_buffer.save_to_file("res/test.wav");
      let mut display = display_sfml::DisplaySFML::new(&beep_buffer);
      display.init();

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