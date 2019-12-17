extern crate intcode;

fn main() {
    let rom = intcode::parse_rom("input");
    println!(
        "9-1:\n{}",
        intcode::Cpu::new(rom.to_vec()).run_with(&vec![1]).unwrap()
    );
    println!(
        "9-2:\n{}",
        intcode::Cpu::new(rom.to_vec()).run_with(&vec![2]).unwrap()
    );
}
