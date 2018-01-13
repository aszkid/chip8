extern crate rand;

use std::time::Duration;
use std::thread;
use std::fs::File;
use std::io::prelude::*;
use rand::Rng;

/**
* http://mattmik.com/files/chip8/mastering/chip8.html
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROGRAM_BASE: u16 = 0x200;

/**
* Some helper functions.
*/
fn add_carry(a: u8, b: u8) -> (u8, u8) {
      let res = (a as u16) + (b as u16);
      let carry = res & 0xFF00;

      if carry > 0 {
            ((res % 256) as u8, 0x1)
      } else {
            (res as u8, 0x0)
      }
}
fn sub_borrow(a: u8, b: u8) -> (u8, u8) {
      let under = b > a;
      let res = if under {
            0xFF - (b - a) + 1
      } else {
            a - b
      };

      (res, if under {0x0} else {0x1})
}

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
                                    0xE0 => println!("Clear screen"),
                                    0xEE => println!("Return"),
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
                        0x8000 => {
                              match instruction & 0x000F {
                                    0x0 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let val = self.load(ry);

                                          self.store(rx, val);
                                          println!("Store V{} into V{}", ry, rx);
                                    },
                                    0x1 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let val = self.load(rx) | self.load(ry);

                                          self.store(rx, val);
                                    },
                                    0x2 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let val = self.load(rx) & self.load(ry);

                                          self.store(rx, val);
                                    },
                                    0x3 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let val = self.load(rx) ^ self.load(ry);

                                          self.store(rx, val);
                                    },
                                    0x4 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let (val, carry) = add_carry(self.load(rx), self.load(ry));

                                          self.store(rx, val);
                                          self.store(0xF, carry);
                                    },
                                    0x5 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let (val, borrow) = sub_borrow(self.load(rx), self.load(ry));

                                          self.store(rx, val);
                                          self.store(0xF, borrow);
                                    },
                                    0x6 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;

                                          let val = self.load(ry);
                                          // extract lsb
                                          self.store(0xF, val & 0x0001);
                                          self.store(rx, val >> 1);
                                    },
                                    0x7 => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;
                                          let (val, borrow) = sub_borrow(self.load(ry), self.load(rx));

                                          self.store(rx, val);
                                          self.store(0xF, borrow);
                                    },
                                    0xE => {
                                          let rx = ((instruction & 0x0F00) >> 8) as usize;
                                          let ry = ((instruction & 0x00F0) >> 4) as usize;

                                          let val = self.load(ry);
                                          // extract msb
                                          self.store(0xF, val >> 7);
                                          self.store(rx, val << 1);
                                    },
                                    _ => println!("Instruction {:x} unimplemented", instruction)
                              }
                        },
                        0x9000 => println!("Skip neq reg"),
                        0xA000 => {
                              let addr = instruction & 0x0FFF;
                              self.index = addr;
                              println!("Store addr. {:x} in register I", addr);
                        },
                        0xB000 => println!("Jump addr"),
                        0xC000 => {
                              let rx = ((instruction & 0x0F00) >> 8) as usize;
                              let mask = (instruction & 0x00FF) as u8;

                              self.store(rx, rand::random::<u8>() & mask);
                        },
                        0xD000 => {
                              let rx = (instruction & 0x0F00) >> 8;
                              let ry = (instruction & 0x00F0) >> 4;
                              let bytes = instruction & 0x000F;
                              println!("Draw sprite at (V{},V{}) = ({},{}) with {} bytes of data starting at I = {:x}", rx, ry, self.load(rx as usize), self.load(ry as usize), bytes, self.index);
                        },
                        0xE000 => println!("Skip key"),
                        0xF000 => {
                              match instruction & 0x00FF {
                                    0x0A => {
                                          let reg = (instruction & 0x0F00) >> 8;
                                          println!("Wait for keypress and store in V{}", reg);
                                    },
                                    0x18 => {
                                          let reg = (instruction & 0x0F00) >> 8;
                                          println!("Set sound timer to value of V{} = {}", reg, self.load(reg as usize));
                                    },
                                    0x29 => {
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
      fn load(&self, reg: usize) -> u8 {
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

      fn dump(&self) {
            println!("======================");
            println!("  REGISTER DUMP");
            println!("======================");
            for i in 0..self.registers.len() {
                  println!(" --> V{:} = {}", i, self.registers[i]);
            }
      }
}

fn main() {

      let mut chip = Chip::new();

      chip.load_rom("roms/rand.rom");
      chip.run();
      chip.dump();
}