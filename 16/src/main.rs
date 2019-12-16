fn parse_input(filename: &str) -> Vec<i64> {
    let input = std::fs::read_to_string(filename).unwrap();
    input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i64)
        .collect()
}

fn cycle_phase(phase: &Vec<i64>, iterations: usize, offset: usize) -> Vec<i64> {
    let mut phase = phase.to_vec();
    let len = phase.len();
    for _ in 0..iterations {
        let mut next_phase: Vec<i64> = vec![0; len];
        // The right half of a phase is a simple sum because of the wave's properties
        // Calculate the left half naively
        if offset < len / 2 {
            for i in offset..len / 2 {
                let base = [0, 1, 0, -1]
                    .iter()
                    .flat_map(|n| std::iter::repeat(n).take(i + 1))
                    .cycle()
                    .skip(1);
                next_phase[i] = phase.iter().zip(base).map(|(a, b)| a * b).sum::<i64>();
            }
        }
        // And then do a rolling sum for the right half, starting from the end and working back
        let start = std::cmp::max(len / 2, offset);
        // Set the starting value to avoid index overflows
        next_phase[len - 1] = phase[len - 1];
        for i in (start..len - 1).rev() {
            next_phase[i] = phase[i] + next_phase[i + 1];
        }
        // Finally grab the last digit of every number
        for n in &mut next_phase {
            *n = n.abs() % 10;
        }
        phase = next_phase;
    }
    phase
}

fn main() {
    let phase = parse_input("input");
    println!(
        "16-1:\n{}",
        cycle_phase(&phase, 100, 0)
            .iter()
            .take(8)
            .flat_map(|i| std::iter::once(std::char::from_digit(*i as u32, 10).unwrap()))
            .collect::<String>()
    );

    // I think this is more readable than a multiplicative fold. Don't judge, thanks.
    let offset: usize = phase
        .iter()
        .take(7)
        .flat_map(|i| std::iter::once(std::char::from_digit(*i as u32, 10).unwrap()))
        .collect::<String>()
        .parse()
        .unwrap();
    let full_phase = phase
        .iter()
        .cycle()
        .take(phase.len() * 10_000)
        .cloned()
        .collect::<Vec<i64>>();
    println!(
        "16-2\n{}",
        cycle_phase(&full_phase, 100, offset)
            .iter()
            .skip(offset)
            .take(8)
            .flat_map(|i| std::iter::once(std::char::from_digit(*i as u32, 10).unwrap()))
            .collect::<String>()
    );
}
