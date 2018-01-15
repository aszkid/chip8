/**
* http://mattmik.com/files/chip8/mastering/chip8.html -- chip8 overview
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM -- chip8 technical reference
* http://stevelosh.com/blog/2016/12/chip8-cpu/ -- a chip8 emulator in commmon lisp
*/
extern crate rand;

use std::time::Duration;
use std::thread;
use std::fs::File;
use std::io::prelude::*;

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROGRAM_BASE: u16 = 0x200;
const DISPLAY_W: usize = 64;
const DISPLAY_H: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_W * DISPLAY_H;

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

pub struct Chip {
      memory: [u8; MEMORY_SIZE],
      registers: [u8; NUM_REGISTERS],
      running: bool,
      stack: [u16; STACK_SIZE],
      stack_pointer: u8,
      program_counter: u16,
      rom: String,
      index: u16,
      display: [u8; DISPLAY_SIZE]
}

impl Chip {
      pub fn new() -> Chip {
            Chip {
                  memory: [0; MEMORY_SIZE],
                  registers: [0; NUM_REGISTERS],
                  running: false,
                  stack: [0; STACK_SIZE],
                  stack_pointer: 0,
                  program_counter: PROGRAM_BASE,
                  rom: String::from(""),
                  index: 0,
                  display: [0; DISPLAY_SIZE]
            }
      }

      fn display_read(&self, x: usize, y: usize) -> u8 {
            self.display[y * DISPLAY_W + x]
      }

      fn set_flag(&mut self, val: u8) {
            self.registers[0xF] = val;
      }
      fn get_flag(&self) -> u8 {
            self.registers[0xF]
      }

      pub fn run(&mut self) {
            while self.running {
                  self.cycle();
                  // TODO; batch sleep for every X cycles to avoid calling `sleep` too many times
                  thread::sleep(Duration::from_millis(2))
            }
      }
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
                                    0xE0 => self.op_clearsrc(),
                                    0xEE => self.op_ret(),
                                    _ => panic!("Execute machine language subroutine unsupported.")
                              }
                        },
                        0x1000 => self.op_jump_imm(instruction),
                        0x2000 => self.op_call(instruction),
                        0x3000 => self.op_se_reg_imm(instruction),
                        0x4000 => self.op_sne_reg_imm(instruction),
                        0x5000 => {
                              if instruction & 0x000F == 0 {
                                    self.op_se_reg_reg(instruction);
                              } else {
                                    panic!("Undefined instrution {:x}", instruction);
                              }
                        },
                        0x6000 => self.op_load_reg_imm(instruction),
                        0x7000 => self.op_add_reg_imm(instruction),
                        0x8000 => {
                              match instruction & 0x000F {
                                    0x0 => self.op_load_reg_reg(instruction),
                                    0x1 => self.op_or(instruction),
                                    0x2 => self.op_and(instruction),
                                    0x3 => self.op_xor(instruction),
                                    0x4 => self.op_add_reg_reg(instruction),
                                    0x5 => self.op_sub_reg_reg(instruction),
                                    0x6 => self.op_shr(instruction),
                                    0x7 => self.op_subn_reg_reg(instruction),
                                    0xE => self.op_shl(instruction),
                                    _ => panic!("Instruction {:x} unimplemented", instruction)
                              }
                        },
                        0x9000 => {
                              if instruction & 0x000F == 0{
                                    self.op_sne_reg_reg(instruction);
                              } else {
                                    panic!("Undefined instruction {:x}", instruction);
                              }
                        },
                        0xA000 => self.op_load_i_imm(instruction),
                        0xB000 => self.op_jump_i_imm_plus(instruction),
                        0xC000 => self.op_rand(instruction),
                        0xD000 => self.op_draw(instruction),
                        0xE000 => {
                              match instruction & 0x00FF {
                                    0x9E => self.op_skp(instruction),
                                    0xA1 => self.op_sknp(instruction),
                                    _ => panic!("Undefined instruction {:x}", instruction)
                              }
                        },
                        0xF000 => {
                              match instruction & 0x00FF {
                                    0x0A => self.op_load_reg_key(instruction),
                                    0x18 => self.op_load_st_reg(instruction),
                                    0x1E => self.op_add_i_reg(instruction),
                                    0x29 => self.op_load_font_reg(instruction),
                                    _ => panic!("Timer stuff")
                              }
                        },
                        _ => panic!("dunno")
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

      pub fn reset(&mut self) {
            self.memory = [0; MEMORY_SIZE];
            self.registers = [0; NUM_REGISTERS];
            self.running = true;
            self.stack = [0; STACK_SIZE];
            self.stack_pointer = 0;
            self.program_counter = PROGRAM_BASE;
            self.index = 0;
            self.display = [0; DISPLAY_SIZE];
      }

      pub fn load_rom(&mut self, rom: &str) {
            self.reset();
            self.rom = rom.to_string();

            let mut file = File::open(rom).unwrap();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();

            for i in 0..contents.len() {
                  self.memory[PROGRAM_BASE as usize + i] = contents[i];
            }
      }

      pub fn dump(&self) {
            println!("======================");
            println!("  REGISTER DUMP");
            println!("======================");
            for i in 0..self.registers.len() {
                  println!(" --> V{:} = {}", i, self.registers[i]);
            }
      }

      /**
       * Instruction implementations.
       */
      fn op_clearsrc(&mut self) {
            self.display = [0; DISPLAY_SIZE];
      }
      fn op_ret(&mut self) {
            println!("Subroutine return");
            // TODO
      }
      fn op_jump_imm(&mut self, instruction: u16) {
            let addr = instruction & 0x0FFF;
            self.program_counter = addr;
      }
      fn op_call(&mut self, instruction: u16) {
            let addr = instruction & 0x0FFF;
            println!("Subroutine call");
            // TODO
      }
      fn op_load_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;
            
            self.store(reg, val);
      }
      fn op_load_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(ry);

            self.store(rx, val);
      }
      fn op_or(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) | self.load(ry);

            self.store(rx, val);
      }
      fn op_and(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) & self.load(ry);

            self.store(rx, val);
      }
      fn op_xor(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) ^ self.load(ry);

            self.store(rx, val);
      }
      fn op_add_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, carry) = add_carry(self.load(rx), self.load(ry));

            self.store(rx, val);
            self.set_flag(carry);
      }
      fn op_sub_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, borrow) = sub_borrow(self.load(rx), self.load(ry));

            self.store(rx, val);
            self.set_flag(borrow);
      }
      fn op_shr(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            let val = self.load(ry);
            // extract lsb
            self.set_flag(val & 0x0001);
            self.store(rx, val >> 1);
      }
      fn op_subn_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, borrow) = sub_borrow(self.load(ry), self.load(rx));

            self.store(rx, val);
            self.set_flag(borrow);
      }
      fn op_shl(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            let val = self.load(ry);
            // extract msb
            self.set_flag(val >> 7);
            self.store(rx, val << 1);
      }
      fn op_load_i_imm(&mut self, instruction: u16) {
            let addr = instruction & 0x0FFF;
            self.index = addr;
      }
      fn op_jump_i_imm_plus(&mut self, instruction: u16) {
            self.program_counter = (instruction & 0x0FFF) + self.load(0x0) as u16;
      }
      fn op_rand(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let mask = (instruction & 0x00FF) as u8;

            self.store(rx, rand::random::<u8>() & mask);
      }
      fn op_draw(&mut self, instruction: u16) {
            let rx = (instruction & 0x0F00) >> 8;
            let ry = (instruction & 0x00F0) >> 4;
            let bytes = instruction & 0x000F;
            // TODO
      }
      fn op_load_reg_key(&mut self, instruction: u16) {
            let reg = (instruction & 0x0F00) >> 8;
            // TODO
      }
      fn op_load_st_reg(&mut self, instruction: u16) {
            let reg = (instruction & 0x0F00) >> 8;
            // TODO
      }
      fn op_add_i_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            self.index += self.load(rx) as u16;
      }
      fn op_load_font_reg(&mut self, instruction: u16) {
            let reg = (instruction & 0x0F00) >> 8;
            // TODO
      }
      fn op_se_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;

            if self.load(reg) == val {
                  self.program_counter += 2;
            }
      }
      fn op_sne_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;

            if self.load(reg) != val {
                  self.program_counter += 2;
            }
      }
      fn op_se_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            if self.load(rx) == self.load(ry) {
                  self.program_counter += 2;
            }
      }
      fn op_add_reg_imm(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8 + self.load(rx);

            self.store(rx, val);
      }
      fn op_sne_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            if self.load(rx) != self.load(ry) {
                  self.program_counter += 2;
            }
      }
      fn op_skp(&mut self, instruction: u16) {
            println!("Skip if key pressed");
      }
      fn op_sknp(&mut self, instruction: u16) {
            println!("Skip if key not pressed");
      }
}