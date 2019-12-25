use std::fs;

fn fuel(mass: i64) -> i64 {
    mass / 3 - 2
}

fn extra_fuel(mass: i64) -> i64 {
    let mass = fuel(mass);
    match mass {
        _ if (mass <= 0) => 0,
        _ => mass + extra_fuel(mass)
    }
}

fn main() {
    let input = fs::read_to_string("input").unwrap();

    let sum: i64 = input.lines().map(|line| {
        fuel(line.parse::<i64>().unwrap())
    }).sum();
    let extra_sum: i64 = input.lines().map(|line| {
        extra_fuel(line.parse::<i64>().unwrap())
    }).sum();

    println!("1-1:\n{}\n1-2:\n{}", sum, extra_sum);
}
