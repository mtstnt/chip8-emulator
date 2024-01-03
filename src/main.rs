mod cpu;
mod graphics;

use std::fs;
use crate::cpu::CPU;

fn main() {
    let filename = "roms/Zero Demo [zeroZshadow, 2007].ch8";
    // let filename = "roms/2-ibm-logo.ch8";
    // let filename = "roms/test_opcode.ch8";
    let rom_contents = fs::read(filename).expect("failed to read rom file");
    let mut cpu = CPU::new(rom_contents);
    cpu.execute();
}
