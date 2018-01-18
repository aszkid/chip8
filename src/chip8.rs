/**
* http://mattmik.com/files/chip8/mastering/chip8.html -- chip8 overview
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM -- chip8 technical reference
* http://stevelosh.com/blog/2016/12/chip8-cpu/ -- a chip8 emulator in commmon lisp
*/
extern crate rand;
extern crate time;

use std::time::Duration;
use std::thread;
use std::fs::File;
use std::io::prelude::*;

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROGRAM_BASE: u16 = 0x200;
pub const DISPLAY_W: usize = 64;
pub const DISPLAY_H: usize = 32;
pub const DISPLAY_SIZE: usize = DISPLAY_W * DISPLAY_H;
const KEYPAD_SIZE: usize = 16;

const FONT_SET: [u8; 80] = [
      0xF0, 0x90, 0x90, 0x90, 0xF0,
      0x20, 0x60, 0x20, 0x20, 0x70,
      0xF0, 0x10, 0xF0, 0x80, 0xF0,
      0xF0, 0x10, 0xF0, 0x10, 0xF0,
      0x90, 0x90, 0xF0, 0x10, 0x10,
      0xF0, 0x80, 0xF0, 0x10, 0xF0,
      0xF0, 0x80, 0xF0, 0x90, 0xF0,
      0xF0, 0x10, 0x20, 0x40, 0x40,
      0xF0, 0x90, 0xF0, 0x90, 0xF0,
      0xF0, 0x90, 0xF0, 0x10, 0xF0,
      0xF0, 0x90, 0xF0, 0x90, 0x90,
      0xE0, 0x90, 0xE0, 0x90, 0xE0,
      0xF0, 0x80, 0x80, 0x80, 0xF0,
      0xE0, 0x90, 0x90, 0x90, 0xE0,
      0xF0, 0x80, 0xF0, 0x80, 0xF0,
      0xF0, 0x80, 0xF0, 0x80, 0x80
];

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
      let not_borrow = a > b;
      let res = if a >= b {
            a - b
      } else {
            0xFF - (b - a) + 1
      };

      (res, if not_borrow {0x1} else {0x0})
}

pub struct Chip {
      pub memory: [u8; MEMORY_SIZE],
      pub registers: [u8; NUM_REGISTERS],
      pub running: bool,
      pub stack: [u16; STACK_SIZE],
      pub stack_pointer: usize,
      pub program_counter: u16,
      pub rom: String,
      rom_size: usize,
      pub index: u16,
      pub display: [bool; DISPLAY_SIZE],
      pub clock: u64,
      pub delay_timer: u8,
      pub sound_timer: u8,
      pub key_pressed: u8,
      pub keypad: [bool; KEYPAD_SIZE],
      wait: u8
}

impl Chip {
      pub fn new() -> Chip {
            let mut c = Chip {
                  memory: [0; MEMORY_SIZE],
                  registers: [0; NUM_REGISTERS],
                  running: false,
                  stack: [0; STACK_SIZE],
                  stack_pointer: 0,
                  program_counter: PROGRAM_BASE,
                  rom: String::from(""),
                  rom_size: 0,
                  index: 0,
                  display: [false; DISPLAY_SIZE],
                  clock: time::precise_time_ns(),
                  delay_timer: 0,
                  sound_timer: 0,
                  key_pressed: 0x10,
                  keypad: [false; KEYPAD_SIZE],
                  wait: 0x10
            };
            c.reset();
            c
      }

      fn display_write_byte(&mut self, x: usize, y: usize, byte: u8) -> bool {
            // TODO:
            // 1 - wrap around
            // 2 - set VF if pixel erased -- DONE
            let mut idx = y * DISPLAY_W + x;
            let mut overlap = false;
            for j in 0..8 {
                  if idx < DISPLAY_SIZE {
                        let mask = 0b00000001 << (7 - j);
                        let old = self.display[idx].clone();
                        self.display[idx] ^= (byte & mask) != 0;
                        if old == true && self.display[idx] == false {
                              overlap = true;
                        }
                        idx += 1;
                  }
            }
            overlap
      }

      fn set_flag(&mut self, val: u8) {
            self.registers[0xF] = val;
      }

      pub fn cycle(&mut self) {
            let high: u16 = self.memory[self.program_counter as usize] as u16;
            let low: u16 = self.memory[(self.program_counter + 1) as usize] as u16;

            let mut instruction: u16 = 0;
            instruction |= high << 8;
            instruction |= low;

            // Delay & sound timers are decreased by 1 at a rate of 60Hz
            let now = time::precise_time_ns();
            if now - self.clock > 16666000 {
                  self.clock = now;
                  if self.delay_timer != 0 {
                        self.delay_timer -= 1;
                  }
                  if self.sound_timer != 0 {
                        self.sound_timer -= 1;
                  }
            }

            println!("Dispatching instruction {:x} at PC = {} (I = {})", instruction, self.program_counter, self.index);
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
                  0xB000 => self.op_jump_imm_plus(instruction),
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
                              0x07 => self.op_load_reg_dt(instruction),
                              0x0A => self.op_load_reg_key(instruction),
                              0x15 => self.op_load_dt_reg(instruction),
                              0x18 => self.op_load_st_reg(instruction),
                              0x1E => self.op_add_i_reg(instruction),
                              0x29 => self.op_load_font_reg(instruction),
                              0x33 => self.op_load_bcd_reg(instruction),
                              0x55 => self.op_store_regs_i(instruction),
                              0x65 => self.op_load_regs_i(instruction),
                              _ => panic!("Undefined instruction {:x}", instruction)
                        }
                  },
                  _ => panic!("dunno")
            }

            if self.program_counter == 4094 {
                  println!("Finished memory!");
                  self.running = false;
                  return
            }
            if self.program_counter - PROGRAM_BASE >= self.rom_size as u16 {
                  println!("Finished ROM!");
                  self.running = false;
                  return
            } 
      }

      fn store(&mut self, reg: usize, val: u8) {
            self.registers[reg] = val;
      }
      fn load(&self, reg: usize) -> u8 {
            self.registers[reg]
      }

      pub fn reset(&mut self) {
            self.memory = [0; MEMORY_SIZE];
            self.memory[0..80].copy_from_slice(&FONT_SET);
            self.registers = [0; NUM_REGISTERS];
            self.running = true;
            self.stack = [0; STACK_SIZE];
            self.stack_pointer = 0;
            self.program_counter = PROGRAM_BASE;
            self.index = 0;
            self.display = [false; DISPLAY_SIZE];
            self.clock = time::precise_time_ns();
            self.delay_timer = 0;
            self.sound_timer = 0;
            self.key_pressed = 0x10;
            self.keypad = [false; KEYPAD_SIZE];
            self.wait = 0x10;
      }

      pub fn load_rom(&mut self, rom: &str) {
            self.reset();
            self.rom = rom.to_string();

            let mut file = File::open(rom).unwrap();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            self.rom_size = contents.len();

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
            println!(" --> DT  = {}", self.delay_timer);
            println!(" --> ST  = {}", self.sound_timer);
            println!(" --> PC  = {}", self.program_counter);
            println!(" --> I   = {}", self.index);
      }

      /**
       * Instruction implementations.
       */
      fn op_clearsrc(&mut self) {
            self.display = [false; DISPLAY_SIZE];

            self.program_counter += 2;
      }
      fn op_ret(&mut self) {
            self.program_counter = self.stack[self.stack_pointer-1]/* + 2*/;
            self.stack_pointer -= 1;
      }
      fn op_jump_imm(&mut self, instruction: u16) {
            let addr = instruction & 0x0FFF;
            self.program_counter = addr;
      }
      fn op_call(&mut self, instruction: u16) {
            if self.stack_pointer >= STACK_SIZE {
                  panic!("Stack overflow!");
            }
            let addr = (instruction & 0x0FFF) as u16;
            self.stack[self.stack_pointer] = self.program_counter + 2;
            self.program_counter = addr;
            self.stack_pointer += 1;
      }
      fn op_load_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;
            
            self.store(reg, val);
            self.program_counter += 2;
      }
      fn op_load_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(ry);

            self.store(rx, val);
            self.program_counter += 2;
      }
      fn op_or(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) | self.load(ry);

            self.store(rx, val);
            self.program_counter += 2;
      }
      fn op_and(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) & self.load(ry);

            self.store(rx, val);
            self.program_counter += 2;
      }
      fn op_xor(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let val = self.load(rx) ^ self.load(ry);

            self.store(rx, val);
            self.program_counter += 2;
      }
      fn op_add_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, carry) = add_carry(self.load(rx), self.load(ry));

            self.store(rx, val);
            self.set_flag(carry);
            self.program_counter += 2;
      }
      fn op_sub_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, borrow) = sub_borrow(self.load(rx), self.load(ry));

            self.store(rx, val);
            self.set_flag(borrow);
            self.program_counter += 2;
      }
      fn op_shr(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            let val = self.load(ry);
            // extract lsb
            self.set_flag(val & 0x0001);
            self.store(rx, val >> 1);
            self.program_counter += 2;
      }
      fn op_subn_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let (val, borrow) = sub_borrow(self.load(ry), self.load(rx));

            self.store(rx, val);
            self.set_flag(borrow);
            self.program_counter += 2;
      }
      fn op_shl(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            let val = self.load(ry);
            // extract msb
            self.set_flag(val >> 7);
            self.store(rx, val << 1);
            self.program_counter += 2;
      }
      fn op_load_i_imm(&mut self, instruction: u16) {
            let addr = instruction & 0x0FFF;
            self.index = addr;
            self.program_counter += 2;
      }
      fn op_jump_imm_plus(&mut self, instruction: u16) {
            self.program_counter = (instruction & 0x0FFF) + self.load(0x0) as u16;
      }
      fn op_rand(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let mask = (instruction & 0x00FF) as u8;

            self.store(rx, rand::random::<u8>() & mask);
            self.program_counter += 2;
      }
      fn op_draw(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;
            let bytes = (instruction & 0x000F) as usize;
            let pos_x = self.load(rx) as usize;
            let pos_y = self.load(ry) as usize;

            let src = self.memory[(self.index as usize)..(self.index as usize + bytes)].to_vec();
            let mut overlap = false;
            for i in 0..src.len() {
                  if self.display_write_byte(pos_x, pos_y + i, src[i]) {
                        overlap = true;
                  }
            }

            self.store(0xF, if overlap {0x1} else {0x0});
            self.program_counter += 2;
      }
      fn op_load_reg_key(&mut self, instruction: u16) {
            self.wait = ((instruction & 0x0F00) >> 8) as u8;
            if self.wait != 0x10 {
                  if self.key_pressed != 0x10 {
                        let wait = self.wait.clone();
                        let key = self.key_pressed.clone();
                        self.store(wait as usize, key as u8);

                        self.wait = 0x10;
                        self.key_pressed = 0x10;
                        self.program_counter += 2;
                  }
            }
      }
      fn op_load_st_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            self.sound_timer = self.load(rx);

            self.program_counter += 2;
      }
      fn op_add_i_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            self.index += self.load(rx) as u16;

            self.program_counter += 2;
      }
      fn op_load_font_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let digit = self.load(rx);
            self.index = (digit as u16) * 5;

            self.program_counter += 2;
      }
      fn op_se_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;

            if self.load(reg) == val {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_sne_reg_imm(&mut self, instruction: u16) {
            let reg = ((instruction & 0x0F00) >> 8) as usize;
            let val = (instruction & 0x00FF) as u8;

            if self.load(reg) != val {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_se_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            if self.load(rx) == self.load(ry) {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_add_reg_imm(&mut self, instruction: u16) {
            use std::num::Wrapping;

            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = Wrapping((instruction & 0x00FF) as u8);
            let result = val + Wrapping(self.load(rx));

            self.store(rx, result.0);
            self.program_counter += 2;
      }
      fn op_sne_reg_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let ry = ((instruction & 0x00F0) >> 4) as usize;

            if self.load(rx) != self.load(ry) {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_skp(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = (self.load(rx) & 0x0F) as usize;
            
            if self.keypad[val] == true {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_sknp(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = (self.load(rx) & 0x0F) as usize;
            
            if self.keypad[val] != true {
                  self.program_counter += 2;
            }

            self.program_counter += 2;
      }
      fn op_load_dt_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            self.delay_timer = self.load(rx);

            self.program_counter += 2;
      }
      fn op_load_reg_dt(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = self.delay_timer;
            self.store(rx, val);

            self.program_counter += 2;
      }
      fn op_load_bcd_reg(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            let val = self.load(rx) as f32;

            let hundreds = (val / 100.0).floor() as u8 % 10;
            let tens = (val / 10.0).floor() as u8 % 10;
            let ones = val as u8 % 10;

            self.memory[self.index as usize]     = hundreds;
            self.memory[(self.index+1) as usize] = tens;
            self.memory[(self.index+2) as usize] = ones;

            self.program_counter += 2;
      }
      fn op_store_regs_i(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            for j in 0..(rx+1) {
                  self.memory[self.index as usize +j] = self.load(j);
            }

            self.program_counter += 2;
      }
      fn op_load_regs_i(&mut self, instruction: u16) {
            let rx = ((instruction & 0x0F00) >> 8) as usize;
            for j in 0..(rx+1) {
                  let val = self.memory[self.index as usize + j];
                  self.store(j, val);
            }

            self.program_counter += 2;
      }
}