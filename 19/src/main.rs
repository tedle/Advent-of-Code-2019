extern crate intcode;

type Position = (i64, i64);

fn check(rom: &Vec<i64>, pos: Position) -> bool {
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    match cpu.run_with(&vec![pos.0, pos.1]) {
        Some(1) => true,
        _ => false,
    }
}

fn main() {
    let rom = intcode::parse_rom("input");

    println!("19-1:");
    let mut count = 0;
    for x in 0..50 {
        for y in 0..50 {
            if check(&rom, (x, y)) {
                count += 1;
            }
        }
    }
    println!("{}", count);
    println!("19-2:");
    let mut origin = (0, 0);
    let length = 99;
    loop {
        if check(&rom, origin)
            && check(&rom, (origin.0, origin.1 + length))
            && check(&rom, (origin.0 + length, origin.1))
        {
            break;
        }
        origin.1 += 1;
        while !check(&rom, (origin.0, origin.1 + length)) {
            origin.0 += 1;
        }
    }
    println!("{}", origin.0 * 10_000 + origin.1);
}
