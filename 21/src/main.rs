extern crate intcode;

fn main() {
    let rom = intcode::parse_rom("input");
    println!("21-1:");
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    let code = "\
        NOT B T\n\
        OR T J\n\
        NOT C T\n\
        OR T J\n\

        NOT A T\n\
        OR T J\n\
        AND D J\n\

        WALK\n\
    ";
    cpu.push(&code.bytes().map(|b| b as i64).collect());
    while let Some(c) = cpu.run() {
        if c > std::u8::MAX as i64 {
            println!("{}", c);
        }
    }

    println!("21-2:");
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    let code = "\
        NOT B T\n\
        OR T J\n\
        NOT C T\n\
        OR T J\n\

        AND G T\n\
        AND E T\n\
        OR H T\n\
        AND T J\n\

        NOT A T\n\
        OR T J\n\
        AND D J\n\

        RUN\n\
    ";
    cpu.push(&code.bytes().map(|b| b as i64).collect());
    while let Some(c) = cpu.run() {
        if c > std::u8::MAX as i64 {
            println!("{}", c);
        }
    }
}
