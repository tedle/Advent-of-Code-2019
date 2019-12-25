use std::fs;

fn parse_opcodes(filename: &str) -> Vec<usize> {
    let input = fs::read_to_string(filename).unwrap();
    input.split(",").map(|op| op.trim().parse().unwrap()).collect()
}

fn run(opcodes: &Vec<usize>, inputs: (usize, usize)) -> usize {
    let mut opcodes = opcodes.to_vec();
    opcodes[1] = inputs.0;
    opcodes[2] = inputs.1;
    let mut stack_pointer: usize = 0;
    loop {
        match opcodes[stack_pointer] {
            1 => {
                let ax = opcodes[stack_pointer+1];
                let bx = opcodes[stack_pointer+2];
                let cx = opcodes[stack_pointer+3];

                stack_pointer += 4;
                opcodes[cx] = opcodes[ax] + opcodes[bx]
            },
            2 => {
                let ax = opcodes[stack_pointer+1];
                let bx = opcodes[stack_pointer+2];
                let cx = opcodes[stack_pointer+3];

                stack_pointer += 4;
                opcodes[cx] = opcodes[ax] * opcodes[bx]
            },
            99 => return opcodes[0],
            op => {
                stack_pointer += 1;
                println!("Unknown op: {}", op);
            }
        }

    }
}

fn main() {
    let opcodes = parse_opcodes("input");
    println!("2-1:");
    println!("{:?}", run(&opcodes, (12, 2)));
    println!("2-2:");
    for a in 0..99 {
        for b in 0..99 {
            if run(&opcodes, (a, b)) == 19690720 {
                println!("{}", 100 * a + b);
            }
        }
    }
}
