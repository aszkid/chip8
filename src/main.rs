use std::time::Duration;
use std::thread;
use std::fs::File;
use std::io::prelude::*;


/**
* http://mattmik.com/files/chip8/mastering/chip8.html
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROGRAM_BASE: u16 = 0x200;

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
                  program_counter: PROGRAM_BASE,
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
            instruction |= high << 8;
            instruction |= low;

            if instruction != 0 {
                  print!("{:x} | ", instruction);
                  match instruction & 0xF000 {
                        0x0000 => println!("Execute / clearscr / return"),
                        0x1000 => println!("Jump"),
                        0x2000 => println!("Execute"),
                        0x3000 => println!("Skip eq"),
                        0x4000 => println!("Skip neq"),
                        0x5000 => println!("Skip eq reg"),
                        0x6000 => println!("Store"),
                        0x7000 => println!("Add"),
                        0x8000 => println!("Store / setlog / ops"),
                        0x9000 => println!("Skip neq reg"),
                        0xA000 => println!("Store I"),
                        0xB000 => println!("Jump addr"),
                        0xC000 => println!("Rnd"),
                        0xD000 => println!("Draw"),
                        0xE000 => println!("Skip key"),
                        0xF000 => println!("Timer / bindec / etc"),
                        _ => println!("dunno")
                  }
            }

            if self.program_counter == 4094 {
                  println!("Finished memory!");
                  self.running = false;
                  return
            }

            self.program_counter += 2;
      }

      fn reset(&mut self) {
            self.memory = [0; MEMORY_SIZE];
            self.registers = [0; NUM_REGISTERS];
            self.running = true;
            self.stack = [0; STACK_SIZE];
            self.stack_pointer = 0;
            self.program_counter = PROGRAM_BASE;
      }

      fn load_rom(&mut self, rom: &str) {
            self.reset();
            self.rom = rom.to_string();

            let mut file = File::open(rom).unwrap();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();

            for i in 0..contents.len() {
                  self.memory[PROGRAM_BASE as usize + i] = contents[i];
            }
      }
}

fn main() {

      let mut chip = Chip::new();

      chip.load_rom("roms/helloworld.rom");
      chip.run();
}