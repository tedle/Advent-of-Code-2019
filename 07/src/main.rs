extern crate intcode;

/// Shamelessly stolen from: https://en.wikipedia.org/wiki/Heap%27s_algorithm
/// In retrospect figuring out a naive algorithm might've been more fun and less effort
fn permutations(input: Vec<i64>) -> Vec<Vec<i64>> {
    let mut output: Vec<Vec<i64>> = vec![];
    fn heaps(k: usize, array: &mut Vec<i64>, out: &mut Vec<Vec<i64>>) {
        if k == 1 {
            out.push(array.to_vec());
            return;
        }
        heaps(k - 1, array, out);
        for i in 0..k - 1 {
            if k % 2 == 0 {
                array.swap(i, k - 1);
            } else {
                array.swap(0, k - 1);
            }
            heaps(k - 1, array, out);
        }
    };
    heaps(input.len(), &mut input.to_vec(), &mut output);
    output
}

fn main() {
    let rom = intcode::parse_rom("input");
    println!("7-1:");
    let mut max_output = 0;
    for inputs in permutations((0..5).collect()) {
        let mut output = 0;
        for i in inputs {
            output = intcode::Cpu::new(rom.to_vec()).run_with(&vec![i, output]).unwrap();
        }
        max_output = std::cmp::max(max_output, output);
    }
    println!("{}", max_output);
    println!("7-2:");
    let mut max_output = 0;
    for inputs in permutations((5..10).collect()) {
        let mut amplifiers: Vec<intcode::Cpu> = vec![intcode::Cpu::new(rom.to_vec()); 5];
        let mut output = 0;
        for i in 0..5 {
            amplifiers[i].push(&vec![inputs[i]]);
        }
        'feedback_loop: loop {
            for i in 0..5 {
                match amplifiers[i].run_with(&vec![output]) {
                    Some(out) => output = out,
                    None => break 'feedback_loop,
                }
            }
        }
        max_output = std::cmp::max(max_output, output);
    }
    println!("{}", max_output);
}
