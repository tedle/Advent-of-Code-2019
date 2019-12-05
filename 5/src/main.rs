use std::fs;

fn parse_opcodes(filename: &str) -> Vec<i64> {
    let input = fs::read_to_string(filename).unwrap();
    input
        .split(",")
        .map(|op| op.trim().parse().unwrap())
        .collect()
}

fn get_parameter(
    modes: &Vec<usize>,
    index: usize,
    stack_pointer: usize,
    opcodes: &Vec<i64>,
) -> i64 {
    let mode = modes.get(index).unwrap_or(&0);
    match mode {
        0 => opcodes[opcodes[stack_pointer + index] as usize],
        1 => opcodes[stack_pointer + index],
        _ => panic!("Unknown parameter mode"),
    }
}

fn run(opcodes: &Vec<i64>, input: i64) -> Vec<i64> {
    let mut opcodes = opcodes.to_vec();
    let mut stack_pointer: usize = 0;
    let mut outputs: Vec<i64> = vec![];
    loop {
        let op = opcodes[stack_pointer] % 100; // Last 2 digits
        let parameter_modes = (opcodes[stack_pointer] / 100)
            .to_string()
            .chars()
            .map(|n| n.to_digit(10).unwrap() as usize)
            .rev()
            .collect::<Vec<_>>(); // Preceeding digits
        stack_pointer += 1;
        match op {
            1 => {
                // add
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);
                let cx = opcodes[stack_pointer + 2];

                stack_pointer += 3;
                opcodes[cx as usize] = ax + bx;
            }
            2 => {
                // mul
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);
                let cx = opcodes[stack_pointer + 2];

                stack_pointer += 3;
                opcodes[cx as usize] = ax * bx;
            }
            3 => {
                // in
                let ax = opcodes[stack_pointer];

                stack_pointer += 1;
                opcodes[ax as usize] = input;
            }
            4 => {
                // out
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);

                stack_pointer += 1;
                outputs.push(ax);
            }
            5 => {
                // jnz
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);

                stack_pointer = match ax {
                    0 => stack_pointer + 2,
                    _ => bx as usize,
                };
            }
            6 => {
                // jz
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);

                stack_pointer = match ax {
                    0 => bx as usize,
                    _ => stack_pointer + 2,
                };
            }
            7 => {
                // lt
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);
                let cx = opcodes[stack_pointer + 2];

                stack_pointer += 3;
                opcodes[cx as usize] = if ax < bx { 1 } else { 0 };
            }
            8 => {
                // eq
                let ax = get_parameter(&parameter_modes, 0, stack_pointer, &opcodes);
                let bx = get_parameter(&parameter_modes, 1, stack_pointer, &opcodes);
                let cx = opcodes[stack_pointer + 2];

                stack_pointer += 3;
                opcodes[cx as usize] = if ax == bx { 1 } else { 0 };
            }
            99 => break,
            _ => {
                println!("Unknown op: {}", op);
            }
        }
    }
    outputs
}

fn main() {
    let opcodes = parse_opcodes("input");
    println!("5-1:");
    println!("{:?}", run(&opcodes, 1));
    println!("5-2:");
    println!("{:?}", run(&opcodes, 5));
}
