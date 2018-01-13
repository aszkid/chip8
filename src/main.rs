use std::time::Duration;
use std::thread;

/**
* http://mattmik.com/files/chip8/mastering/chip8.html
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;

struct Chip {
      memory: [u8; MEMORY_SIZE],
      registers: [u8; NUM_REGISTERS],
      running: bool,
      stack: [u16; STACK_SIZE],
      stack_pointer: u8,
      program_counter: u16,
      rom: String
}

impl Chip {
      fn new() -> Chip {
            Chip {
                  memory: [0; MEMORY_SIZE],
                  registers: [0; NUM_REGISTERS],
                  running: false,
                  stack: [0; STACK_SIZE],
                  stack_pointer: 0,
                  program_counter: 0x200,
                  rom: String::from("")
            }
      }

      fn set_flag(&mut self, val: u8) {
            self.registers[0xF] = val;
      }
      fn get_flag(&self) -> u8 {
            self.registers[0xF]
      }

      fn run(&mut self) {
            while self.running {
                  self.cycle();
                  // TODO; batch sleep for every X cycles to avoid calling `sleep` too many times
                  thread::sleep(Duration::from_millis(2))
            }
      }
      // private function, in theory
      fn cycle(&mut self) {
            let high: u16 = self.memory[self.program_counter as usize] as u16;
            let low: u16 = self.memory[(self.program_counter + 1) as usize] as u16;

            let mut instruction: u16 = 0;
            instruction |= 0xFF00 & high;
            instruction |= 0x00FF & low;

            println!("instruction: {:x} at {:x}", instruction, self.program_counter);

            if self.program_counter == 4094 {
                  println!("Finished memory!");
                  self.running = false;
            }

            self.program_counter += 2;
      }

      fn reset(&mut self) {
            self.memory = [0; MEMORY_SIZE];
            self.registers = [0; NUM_REGISTERS];
            self.running = true;
            self.stack = [0; STACK_SIZE];
            self.stack_pointer = 0;
            self.program_counter = 0x200;
      }

      fn load_rom(&mut self, rom: &str) {
            self.reset();
            self.rom = rom.to_string();
      }
}

fn main() {

      let mut chip = Chip::new();
      chip.set_flag(231);

      println!("Program counter: {}", chip.program_counter);
      println!("Flag: {}", chip.get_flag());
      println!("ROM: {}", chip.rom);

      chip.load_rom("test.bin");

      println!("Program counter: {}", chip.program_counter);
      println!("Flag: {}", chip.get_flag());
      println!("ROM: {}", chip.rom);

      chip.run();
}