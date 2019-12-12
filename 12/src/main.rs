use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Vec3 {
    x: i64,
    y: i64,
    z: i64,
}

impl Vec3 {
    fn new() -> Vec3 {
        Vec3 { x: 0, y: 0, z: 0 }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Moon {
    pos: Vec3,
    velocity: Vec3,
}

impl Moon {
    fn new() -> Moon {
        Moon {
            pos: Vec3::new(),
            velocity: Vec3::new(),
        }
    }

    fn energy(&self) -> i64 {
        let potential = self.pos.x.abs() + self.pos.y.abs() + self.pos.z.abs();
        let kinetic = self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs();
        potential * kinetic
    }
}

fn parse_input(filename: &str) -> Vec<Moon> {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut moons: Vec<Moon> = vec![];
    fn find_between<'a>(search: &'a str, start: &str, end: &str) -> &'a str {
        let start_i = search.find(start).unwrap() + start.len();
        let end_i = search[start_i..].find(end).unwrap() + start_i;
        &search[start_i..end_i]
    }
    for line in input.lines() {
        let mut moon = Moon::new();
        moon.pos.x = find_between(line, "x=", ",").parse().unwrap();
        moon.pos.y = find_between(line, "y=", ",").parse().unwrap();
        moon.pos.z = find_between(line, "z=", ">").parse().unwrap();
        moons.push(moon);
    }
    moons
}

fn step(moons: &Vec<Moon>) -> Vec<Moon> {
    let mut next_state = moons.to_vec();
    fn velocity_cmp(a: &i64, b: &i64) -> i64 {
        match a.partial_cmp(&b) {
            Some(std::cmp::Ordering::Equal) => 0,
            Some(std::cmp::Ordering::Less) => 1,
            Some(std::cmp::Ordering::Greater) => -1,
            None => panic!("Unexpected order"),
        }
    }
    for (i, moon) in next_state.iter_mut().enumerate() {
        for (j, neighbour) in moons.iter().enumerate() {
            if i == j {
                continue;
            }
            moon.velocity.x += velocity_cmp(&moon.pos.x, &neighbour.pos.x);
            moon.velocity.y += velocity_cmp(&moon.pos.y, &neighbour.pos.y);
            moon.velocity.z += velocity_cmp(&moon.pos.z, &neighbour.pos.z);
        }
    }
    for moon in &mut next_state {
        moon.pos.x += moon.velocity.x;
        moon.pos.y += moon.velocity.y;
        moon.pos.z += moon.velocity.z;
    }
    next_state
}

fn gcd(a: usize, b: usize) -> usize {
    match b {
        0 => a,
        _ => gcd(b, a % b),
    }
}

fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}

fn lcm_slice(factors: &[usize]) -> usize {
    match factors.len() {
        0 => 0,
        1 => factors[0],
        _ => lcm(factors[0], lcm_slice(&factors[1..])),
    }
}

macro_rules! find_axis_loop {
    ($moons:ident,$($axis:ident),+) => {{
        let mut iteration_counts = Vec::<usize>::new();
        $(
            let mut moons = $moons.to_vec();
            let mut iteration_cache: HashSet<Vec<i64>> = HashSet::new();
            loop {
                let iteration = moons
                    .iter()
                    .flat_map(|m| vec![m.pos.$axis, m.velocity.$axis])
                    .collect::<Vec<i64>>();

                if iteration_cache.get(&iteration).is_some() {
                    break;
                };

                iteration_cache.insert(iteration);
                moons = step(&moons);
            }
            iteration_counts.push(iteration_cache.len());
        )+
        iteration_counts
    }};
}

fn main() {
    let moons = parse_input("input");

    println!("12-1:");
    let mut moon_energy = moons.to_vec();
    for _ in 0..1000 {
        moon_energy = step(&moon_energy);
    }
    println!("{}", moon_energy.iter().map(|m| m.energy()).sum::<i64>());
    println!("12-2:");
    let iteration_counts = find_axis_loop!(moons, x, y, z);
    println!("{}", lcm_slice(&iteration_counts));
}
