extern crate intcode;

fn str_to_input(input: &str) -> Vec<i64> {
    input.bytes().map(|b| b as i64).collect()
}

#[allow(dead_code)]
fn run_interactive(rom: &Vec<i64>, initial_input: &str) {
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    cpu.push(&str_to_input(initial_input));
    loop {
        match cpu.run() {
            c @ Some(0..=255) => {
                let c = c.unwrap() as u8 as char;
                print!("{}", c);
                if c == '?' && cpu.inputs.len() == 0 {
                    print!("\n");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to get input");
                    match input.as_str().trim() {
                        "exit" => break,
                        "" => (),
                        _ => cpu.push(&str_to_input(input.as_str())),
                    }
                }
            }
            Some(out) => panic!("Unexpected output {}", out),
            None => break,
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");
    let solution = [
        "south", "east",
        "take space heater",
        "west", "north", "west", "north", "east", "south",
        "take asterisk",
        "south",
        "take klein bottle",
        "north", "north", "west", "north",
        "take astronaut ice cream",
        "south", "south", "south", "west", "south",
    ]
    .join("\n")
        + "\n";

    println!("25:");
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    let mut output = String::new();
    cpu.push(&str_to_input(solution.as_str()));
    loop {
        match cpu.run() {
            c @ Some(0..=255) => output.push(c.unwrap() as u8 as char),
            Some(out) => panic!("Unexpected output {}", out),
            None => break,
        }
    }
    for word in output.lines().last().unwrap().split_ascii_whitespace() {
        match word.parse::<i64>() {
            Ok(num) => println!("{}", num),
            _ => {}
        }
    }
}
