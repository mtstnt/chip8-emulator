use std::fs;

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
    nnn: u16,
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
            nnn: 0u16 | (second_nibble as u16) << 8 | (third_nibble as u16) << 4 | (fourth_nibble as u16)
        }
    }
}

#[derive(Debug)]
pub struct CPU {
    instructions: Vec<u8>,
    pc: usize,
}

impl CPU {
    pub fn new(rom_contents: Vec<u8>) -> Self {
        CPU {
            instructions: rom_contents,
            pc: 0,
        }
    }

    pub fn execute(&mut self) {
        while self.pc < self.instructions.len() {
            let instruction_bytes = &self.instructions[self.pc..(self.pc + INSTRUCTION_SIZE)];
            self.pc += INSTRUCTION_SIZE;

            // Decode and get the parts of the instructions.
            let instruction = Instruction::new(instruction_bytes[0], instruction_bytes[1]);
            self.process_instruction(instruction);
        }
    }

    fn process_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction { first: 0x0, x: 0x0, y: 0xE, n: 0x0, .. } => println!("CLEAR SCREEN!!!"),
            Instruction { first: 0x1, nnn, .. } => println!("JUMP TO {:03X?}", nnn),
            Instruction { first: 0x6, x, nn, .. } => println!("SET REGISTER V{:01X?} WITH VALUE {:02X?}", x, nn),
            Instruction { first: 0x7, x, nn, ..} => println!("ADD VALUE {:02X?} TO REGISTER V{:01X?}. NO CARRY EFFECTS HERE.", nn, x),
            // Math ops.
            Instruction { first: 0x8, x, y, n, .. } => {
                match n {
                    0x0 => println!("SET VALUE V{:01X?} WITH VALUE OF V{:01X?}", x, y),
                    0x1 => println!("V{:01x?} | V{:01x?}. SET TO V{:01X?}", x, y, x),
                    0x2 => println!("V{:01X?} & V{:01X?}. SET TO V{:01X?}", x, y, x),
                    0x3 => println!("V{:01X?} ^ V{:01X?}. SET TO V{:01X?}", x, y, x),
                    0x4 => println!("V{:01X?} + V{:01X?}. SET TO V{:01X?} WITH CARRY TO VF", x, y, x),
                    0x5 => println!("V{:01X?} - V{:01X?}. SET TO V{:01X?} WITH CARRY.", x, y, x),
                    0x7 => println!("V{:01X?} - V{:01X?}. SET TO V{:01X?} WITH CARRY.", x, y, y),
                    0x6 => println!("SET VALUE OF V{:01X} FROM V{:01X} AND SHIFT RIGHT BY 1 BIT", x, y),
                    0xE => println!("SET VALUE OF V{:01X} FROM V{:01X} AND SHIFT LEFT BY 1 BIT", x, y),
                    _ => (),
                }
            },
            Instruction { first: 0xA, nnn, .. } => println!("SET INDEX REGISTER TO {:03X?}", nnn),
            Instruction { first: 0xB, nnn, .. } => println!("JUMP WITH OFFSET {:03X?}", nnn),
            Instruction { first: 0xC, x, nn, .. } => println!("GENERATE RANDOM NUMBER THEN ANDS IT WITH {:02X?} AND PUTS IT IN V{:01X?}", x, nn),
            Instruction { first: 0xD, x, y, n, ..} => println!("DISPLAY {:01X?} PIXELS SPRITE FROM I REGISTER WITH X IN V{:01X?} AND Y IN V{:01X?}", n, x, y),

            // Skip if key.
            Instruction { first: 0xE, x, y: 0x9, n: 0x1, .. } => println!("SKIP 1 INSTRUCTION IF KEY IN V{:01X?} IS PRESSED.", x),
            Instruction { first: 0xE, x, y: 0xA, n: 0x1, .. } => println!("SKIP 1 INSTRUCTION IF KEY IN V{:01X?} IS NOT PRESSED.", x),

            // Timers
            Instruction { first: 0xF, x, y: 0x0, n: 0x7, .. } => println!("SET VALUE OF V{:01X?} TO DELAY TIMER", x),
            Instruction { first: 0xF, x, y: 0x1, n: 0x5, .. } => println!("SET DELAY TIMER TO VALUE IN V{:01X?}", x),
            Instruction { first: 0xF, x, y: 0x1, n: 0x8, .. } => println!("SET SOUND TIMER TO VALUE IN V{:01X?}", x),

            Instruction { first: 0xF, x, y: 0x1, n: 0xE, .. } => println!("ADD VALUE V{:01X?} AND ADD TO REGISTER I", x),
            Instruction { first: 0xF, x, y: 0x0, n: 0xA, .. } => println!("WAIT FOR KEY INPUT AND SET TO V{:01X?}", x),
            Instruction { first: 0xF, x, y: 0x2, n: 0x9, .. } => println!("SET INDEX REGISTER I TO V{:01X?} TO CHARACTER IN MEMORY", x),
            Instruction { first: 0xF, x, y: 0x3, n: 0x3, .. } => println!("CONVERT VALUE OF V{:01X?} TO 3 DECIMAL DIGITS, STORING AT INDEX REGISTER I", x),
            Instruction { first: 0xF, x, y: 0x5, n: 0x5, .. } => println!("STORE AND LOAD MEMORY 1 (TBD). REGISTER V{:01X}", x),
            Instruction { first: 0xF, x, y: 0x6, n: 0x5, .. } => println!("STORE AND LOAD MEMORY 2 (TBD). REGISTER V{:01X}", x),
            _ => {
                println!("INSTRUCTION BYTE {:01X?} NOT FOUND!", instruction.nibbles);
            }
        }
    }
}

fn main() {
    // let filename = "roms/Zero Demo [zeroZshadow, 2007].ch8";
    let filename = "roms/2-ibm-logo.ch8";

    let rom_contents = fs::read(filename).expect("failed to read rom file");

    let mut cpu = CPU::new(rom_contents);
    cpu.execute();
}
