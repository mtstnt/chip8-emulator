use std::{os, io};

use crate::graphics::Graphics;

use piston_window::*;
use rand::Rng;

const INSTRUCTION_SIZE: usize = 2;

#[derive(Debug)]
struct Instruction {
    nibbles: [u8; 4],

    // Separated into other sections.
    first: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: usize,
}

impl Instruction {
    pub fn new(first: u8, second: u8) -> Self {
        let first_nibble = first >> 4;
        let second_nibble = first & 15;
        let third_nibble = second >> 4;
        let fourth_nibble = second & 15;

        Instruction {
            nibbles: [first_nibble, second_nibble, third_nibble, fourth_nibble],
            first: first_nibble,
            x: second_nibble,
            y: third_nibble,
            n: fourth_nibble,
            nn: third_nibble << 4 | fourth_nibble,
            nnn: 0usize | (second_nibble as usize) << 8 | (third_nibble as usize) << 4 | (fourth_nibble as usize)
        }
    }
}

// TODO: Use this instead of usize.
// Location type, will be 12 bits max.
pub type MemoryAddr = usize;

pub struct CPU {
    pc: usize,
    registers: [u8; 0xF],
    index_register: usize,

    // Timers. to move to other module.
    delay_timer: u8,
    sound_timer: u8,

    // Memory
    memory: [u8; 4096],

    // SP
    stack: Vec<usize>,

    // Graphics and window manager.
    graphics: Graphics,
}

impl CPU {
    pub fn new(rom_contents: Vec<u8>) -> Self {
        let mut memory: [u8; 4096] = [0x0; 4096];

        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for i in 0..fonts.len() {
            memory[i] = fonts[i];
        }

        let mut co = 0;
        rom_contents.iter().for_each(|e| {
            memory[0x200 + co] = *e;
            co += 1;
        });

        CPU {
            pc: 0x200,
            registers: [0x0; 0xF],
            index_register: 0x0,
            sound_timer: 0x0,
            delay_timer: 0x0,
            memory,
            stack: vec![],
            graphics: Graphics::new(),
        }
    }

    pub fn execute(&mut self) {
        loop {
            let instr = &self.memory[self.pc..(self.pc + INSTRUCTION_SIZE)];
            self.process_instruction(Instruction::new(instr[0], instr[1]));
            if self.graphics.render_window().is_none() {
                println!("Terminated successfully.");
                break;
            }
        }
    }

    fn process_instruction(&mut self, instruction: Instruction) {
        match instruction {
            // Clear screen
            Instruction { first: 0x0, x: 0x0, y: 0xE, n: 0x0, .. } => self.handle_clear_screen(),

            // Subroutines.
            Instruction { first: 0x0, x: 0x0, y: 0xE, n: 0xE, .. } => self.handle_return(),
            Instruction { first: 0x0, nnn, .. } => self.handle_call_subroutine(nnn),

            // Jump
            Instruction { first: 0x1, nnn, .. } => self.handle_jump(nnn),

            // Set register with literal
            Instruction { first: 0x6, x, nn, .. } => self.handle_set_register_literal(x, nn),

            // Add register with literal
            Instruction { first: 0x7, x, nn, ..} => self.handle_add_into_register(x, nn),

            // Skips
            Instruction { first: 0x3, x, nn, .. } => self.handle_skip_on_equal(x, nn),
            Instruction { first: 0x4, x, nn, .. } => self.handle_skip_on_not_equal(x, nn),
            Instruction { first: 0x5, x, y, n: 0, .. } => self.handle_skip_on_equal_register(x, y),
            Instruction { first: 0x9, x, y, n: 0, .. } => self.handle_skip_on_not_equal_register(x, y),

            // Math ops.
            Instruction { first: 0x8, x, y, n, .. } => {
                match n {
                    0x0 => self.handle_set_register(x, y),
                    0x1 => self.handle_binary_or(x, y),
                    0x2 => self.handle_binary_and(x, y),
                    0x3 => self.handle_binary_xor(x, y),
                    0x4 => self.handle_add_register_with_carry(x, y),
                    0x5 => self.handle_subtraction(x, y, x),
                    0x7 => self.handle_subtraction(y, x, y),
                    0x6 => self.handle_shift(x, y, true),
                    0xE => self.handle_shift(x, y, false),
                    _ => (),
                }
            },

            Instruction { first: 0xA, nnn, .. } => self.handle_set_index_register(nnn),
            Instruction { first: 0xB, nnn, .. } => self.handle_jump_with_offset(nnn),
            Instruction { first: 0xC, x, nn, .. } => self.handle_generate_random_number(x, nn),
            Instruction { first: 0xD, x, y, n, ..} => self.handle_display(n, x, y),

            // Skip if key.
            Instruction { first: 0xE, x, y: 0x9, n: 0x1, .. } => self.handle_skip_on_key(x, true),
            Instruction { first: 0xE, x, y: 0xA, n: 0x1, .. } => self.handle_skip_on_key(x, false),

            // Timers
            Instruction { first: 0xF, x, y: 0x0, n: 0x7, .. } => self.handle_get_delay_timer(x),
            Instruction { first: 0xF, x, y: 0x1, n: 0x5, .. } => self.handle_set_delay_timer(x),
            Instruction { first: 0xF, x, y: 0x1, n: 0x8, .. } => self.handle_set_sound_timer(x),

            Instruction { first: 0xF, x, y: 0x1, n: 0xE, .. } => self.handle_add_into_index_register(x),
            Instruction { first: 0xF, x, y: 0x0, n: 0xA, .. } => self.handle_wait_for_key(x),
            Instruction { first: 0xF, x, y: 0x2, n: 0x9, .. } => self.handle_set_index_register_to_addr(x),
            Instruction { first: 0xF, x, y: 0x3, n: 0x3, .. } => self.handle_number_division(x),
            Instruction { first: 0xF, x, y: 0x5, n: 0x5, .. } => self.handle_store_memory(x),
            Instruction { first: 0xF, x, y: 0x6, n: 0x5, .. } => self.handle_load_memory(x),

            _ => {
                println!("INSTRUCTION BYTE {:01X?} NOT FOUND!", instruction.nibbles);
                self.pc += INSTRUCTION_SIZE;
            }
        }
    }

    fn handle_clear_screen(&mut self) {
        println!("CLS");
        self.graphics.clear_pixels();
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_jump(&mut self, jump_addr: usize) {
        // println!("JMP {:03X?}", jump_addr);
        self.pc = jump_addr;
    }

    fn handle_return(&mut self) {
        println!("RET");
        let prev_pc = self.stack.pop().expect("stack is empty!");
        self.pc = prev_pc;
    }

    fn handle_call_subroutine(&mut self, location: usize) {
        println!("CALL {:03X?}", location);
        self.stack.push(self.pc + INSTRUCTION_SIZE);
        self.pc = location;
    }

    fn handle_set_register_literal(&mut self, register: u8, value: u8) {
        println!("SET {:02X?} ({}) V{:01X?}", value, value, register);
        self.registers[register as usize] = value;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_add_into_register(&mut self, register: u8, value: u8) {
        println!("ADD {:02X?} ({}) TO REGISTER V{:01X?}. NO CARRY EFFECTS HERE.", value, value, register);
        self.registers[register as usize] = u8::wrapping_add(self.registers[register as usize], value);
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_skip_on_equal(&mut self, register: u8, value: u8) {
        // println!("SKIP IF V{:01X?} EQUALS TO {:02X?}", register, value);
        self.pc += INSTRUCTION_SIZE;
        if self.registers[register as usize] == value {
            self.pc += INSTRUCTION_SIZE;
        }
    }

    fn handle_skip_on_not_equal(&mut self, register: u8, value: u8) {
        // println!("SKIP IF V{:01X?} IS NOT EQUAL TO {:02X?}", register, value);
        self.pc += INSTRUCTION_SIZE;
        if self.registers[register as usize] != value {
            self.pc += INSTRUCTION_SIZE;
        }
    }

    fn handle_skip_on_equal_register(&mut self, register1: u8, register2: u8) {
        // println!("SKIP IF V{:01X?} IS EQUAL TO V{:02X?}", register1, register2);
        self.pc += INSTRUCTION_SIZE;
        if self.registers[register1 as usize] == self.registers[register2 as usize] {
            self.pc += INSTRUCTION_SIZE;
        }
    }

    fn handle_skip_on_not_equal_register(&mut self, register1: u8, register2: u8) {
        // println!("SKIP IF V{:01X?} IS NOT EQUAL TO V{:01X?}", register1, register2);
        self.pc += INSTRUCTION_SIZE;
        if self.registers[register1 as usize] != self.registers[register2 as usize] {
            self.pc += INSTRUCTION_SIZE;
        }
    }

    fn handle_set_register(&mut self, register1: u8, register2: u8) {
        println!("SET V{:01X?} <= V{:01X?}", register1, register2);
        self.registers[register1 as usize] = self.registers[register2 as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_binary_or(&mut self, register1: u8, register2: u8) {
        // println!("V{:01x?} | V{:01x?}. SET TO V{:01X?}", register1, register2, register1);
        self.registers[register1 as usize] |= self.registers[register2 as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_binary_and(&mut self, register1: u8, register2: u8) {
        // println!("V{:01X?} & V{:01X?}. SET TO V{:01X?}", register1, register2, register1);
        self.registers[register1 as usize] &= self.registers[register2 as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_binary_xor(&mut self, register1: u8, register2: u8) {
        // println!("V{:01X?} ^ V{:01X?}. SET TO V{:01X?}", register1, register2, register1);
        self.registers[register1 as usize] ^= self.registers[register2 as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_add_register_with_carry(&mut self, register1: u8, register2: u8) {
        // println!("V{:01X?} + V{:01X?}. SET TO V{:01X?} WITH CARRY TO VF", register1, register2, register1);
        let reg1 = self.registers[register1 as usize];
        let reg2 = self.registers[register2 as usize];

        let result: usize = reg1 as usize + reg2 as usize;
        let mut resultu8: u8 = 0x0;

        if result > 0xFF {
            resultu8 = result as u8;
            self.registers[0xF - 1] = 1;
        }
        self.registers[register1 as usize] = resultu8;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_subtraction(&mut self, register1: u8, register2: u8, store_register: u8) {
        // println!("V{:01X?} + V{:01X?}. SET TO V{:01X?} WITH CARRY TO VF", register1, register2, store_register);
        let reg1 = self.registers[register1 as usize];
        let reg2 = self.registers[register2 as usize];

        if reg1 >= reg2 {
            self.registers[0xF - 1] = 1;
        } else {
            self.registers[0xF - 1] = 0;
        }
        self.registers[store_register as usize] = u8::wrapping_sub(reg1, reg2);
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_shift(&mut self, register1: u8, _: u8, direction_right: bool) {
        // Using modern CHIP8 specs. We ignore Y completely.
        if direction_right {
            // println!("SHIFT RIGHT V{:01X?} BY 1 BIT", register1);
            self.registers[register1 as usize] >>= 1;
        } else {
            // println!("SHIFT LEFT V{:01X?} BY 1 BIT", register1);
            self.registers[register1 as usize] <<= 1;
        }
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_set_index_register(&mut self, value: usize) {
        println!("LD {:03X?} ({}) I", value, value);
        self.index_register = value;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_jump_with_offset(&mut self, location: usize) {
        // println!("JUMP WITH OFFSET {:03X?}", location);
        self.pc += location + self.registers[0] as usize;
    }

    fn handle_generate_random_number(&mut self, register: u8, value: u8) {
        // println!("GENERATE RANDOM NUMBER THEN ANDS IT WITH {:02X?} AND PUTS IT IN V{:01X?}", register, value);
        let mut rnd = rand::thread_rng();
        self.registers[register as usize] = rnd.gen::<u8>() & value;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_display(&mut self, sprite_size: u8, x_register_source: u8, y_register_source: u8) {
        println!("DRW V{:01X?} V{:01X?} {:01X?} ", x_register_source, y_register_source, sprite_size);

        let x = self.registers[x_register_source as usize];
        let y = self.registers[y_register_source as usize];

        for sprite_row in 0..sprite_size {
            let byte = self.memory[self.index_register + sprite_row as usize];
            for sprite_col in 0..8 {
                let sprite_value = (byte & (1 << 7 - sprite_col)) != 0;

                let xpos = (x + sprite_col as u8) % 64;
                let ypos = (y + sprite_row as u8) % 32;

                let old_value = self.graphics.get_pixel(xpos, ypos);
                let xor_value = old_value ^ sprite_value;

                // If any pixel is erased, set VF to 1, else 0.
                if old_value && !sprite_value {
                    self.registers[0xF - 1] = 1;
                } else {
                    self.registers[0xF - 1] = 0;
                }

                self.graphics.set_pixel(xpos, ypos, xor_value);
                // println!("{:?}, {}", (xpos, ypos), xor_value);
            }
        }
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_skip_on_key(&mut self, register: u8, should_be_pressed: bool) {
        if should_be_pressed {
            // println!("SKIP 1 INSTRUCTION IF KEY IN V{:01X?} IS PRESSED.", register);
            // TODO: Handle check if key is pressed.
        } else {
            // println!("SKIP 1 INSTRUCTION IF KEY IN V{:01X?} IS NOT PRESSED.", register);
            // TODO: Handle check if key is not pressed.
        }
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_get_delay_timer(&mut self, register: u8) {
        // println!("SET VALUE OF V{:01X?} TO VALUE OF DELAY TIMER", register);
        self.registers[register as usize] = self.delay_timer;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_set_delay_timer(&mut self, register: u8) {
        // println!("SET DELAY TIMER TO VALUE IN V{:01X?}", register);
        self.delay_timer = self.registers[register as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_set_sound_timer(&mut self, register: u8) {
        // println!("SET SOUND TIMER TO VALUE IN V{:01X?}", register);
        self.sound_timer = self.registers[register as usize];
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_add_into_index_register(&mut self, register: u8) {
        // println!("ADD VALUE V{:01X?} AND ADD TO REGISTER I", register);
        self.index_register += self.registers[register as usize] as usize;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_wait_for_key(&mut self, register: u8) {
        // println!("WAIT FOR KEY INPUT AND SET TO V{:01X?}", register);
        // TODO: Implement input key checker.
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_set_index_register_to_addr(&mut self, register: u8) {
        // println!("SET INDEX REGISTER I TO V{:01X?} TO CHARACTER IN MEMORY", register);
        self.index_register = self.registers[register as usize] as usize;
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_number_division(&mut self, register: u8) {
        // println!("CONVERT VALUE OF V{:01X?} TO 3 DECIMAL DIGITS, STORING AT INDEX REGISTER I", register);
        let mut reg = self.registers[register as usize];
        let third = reg % 10;
        reg /= 10;
        let second = reg % 10;
        reg /= 10;
        let first = reg;

        let index_addr = self.index_register;
        self.memory[index_addr] = first;
        self.memory[index_addr + 1] = second;
        self.memory[index_addr + 2] = third;

        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_store_memory(&mut self, register: u8) {
        // println!("STORE MEMORY FROM V0 TO V{:01X?} TO INDEX REGISTER", register);
        let mut addr = self.index_register;
        for (_, value) in self.registers[0..=(register as usize)].iter().take(register as usize + 1).enumerate() {
            self.memory[addr] = *value;
            addr += 1;
        }
        self.pc += INSTRUCTION_SIZE;
    }

    fn handle_load_memory(&mut self, register: u8) {
        // println!("LOAD MEMORY FROM INDEX REGISTER TO V0 TO V{:01X?}", register);
        let mut addr = self.index_register;
        for i in 0..self.registers.len() {
            // TODO: Store to memory.
            self.registers[i] = self.memory[addr];
            addr += 1;
        }
        self.pc += INSTRUCTION_SIZE;
    }
}