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
      rom: String,
      index: u16
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
                  rom: String::from(""),
                  index: 0
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
                  match instruction & 0xF000 {
                        0x0000 => {
                              match instruction {
                                    0x00E0 => println!("Clear screen"),
                                    0x00EE => println!("Return"),
                                    _ => println!("Execute machine language subroutine unsupported.")
                              }
                        },
                        0x1000 => {
                              let addr = instruction & 0x0FFF;
                              self.program_counter = addr;
                              println!("Jump to physical address {:x}", addr);
                        },
                        0x2000 => {
                              let addr = instruction & 0x0FFF;
                              println!("Execute subroutine at {:x}", addr);
                        },
                        0x3000 => println!("Skip eq"),
                        0x4000 => println!("Skip neq"),
                        0x5000 => println!("Skip eq reg"),
                        0x6000 => {
                              let reg = ((instruction & 0x0F00) >> 8) as usize;
                              let val = (instruction & 0x00FF) as u8;
                              self.store(reg, val);
                              println!("Store {} in V{}", val, reg);
                        },
                        0x7000 => println!("Add"),
                        0x8000 => println!("Store / setlog / ops"),
                        0x9000 => println!("Skip neq reg"),
                        0xA000 => {
                              let addr = instruction & 0x0FFF;
                              self.index = addr;
                              println!("Store addr. {:x} in register I", addr);
                        },
                        0xB000 => println!("Jump addr"),
                        0xC000 => println!("Rnd"),
                        0xD000 => {
                              let reg_x = (instruction & 0x0F00) >> 8;
                              let reg_y = (instruction & 0x00F0) >> 4;
                              let bytes = instruction & 0x000F;
                              println!("Draw sprite at (V{},V{}) = ({},{}) with {} bytes of data starting at I = {:x}", reg_x, reg_y, self.load(reg_x as usize), self.load(reg_y as usize), bytes, self.index);
                        },
                        0xE000 => println!("Skip key"),
                        0xF000 => {
                              match instruction & 0x00FF {
                                    0x000A => {
                                          let reg = (instruction & 0x0F00) >> 8;
                                          println!("Wait for keypress and store in V{}", reg);
                                    },
                                    0x0018 => {
                                          let reg = (instruction & 0x0F00) >> 8;
                                          println!("Set sound timer to value of V{} = {}", reg, self.load(reg as usize));
                                    },
                                    0x0029 => {
                                          let reg = (instruction & 0x0F00) >> 8;
                                          println!("Set I to sprite mem. address for digit in V{} = {}", reg, self.load(reg as usize));
                                    },
                                    _ => println!("Timer stuff")
                              }
                        },
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

      fn store(&mut self, reg: usize, val: u8) {
            self.registers[reg] = val;
      }
      fn load(&mut self, reg: usize) -> u8 {
            self.registers[reg]
      }

      fn reset(&mut self) {
            self.memory = [0; MEMORY_SIZE];
            self.registers = [0; NUM_REGISTERS];
            self.running = true;
            self.stack = [0; STACK_SIZE];
            self.stack_pointer = 0;
            self.program_counter = PROGRAM_BASE;
            self.index = 0;
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