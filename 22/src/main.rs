extern crate num;
use num::bigint::BigInt;
use num::{Signed, ToPrimitive};

#[derive(Debug, Clone)]
enum DealTechnique {
    NewStack,
    Cut(BigInt),
    Increment(BigInt),
}

impl DealTechnique {
    fn apply(&self, deck: &Vec<BigInt>) -> Vec<BigInt> {
        match self {
            DealTechnique::NewStack => deck.iter().rev().cloned().collect(),
            DealTechnique::Cut(index) => {
                let slice_point = if *index > BigInt::from(0) {
                    index.clone()
                } else {
                    deck.len() - index.abs()
                }
                .to_usize()
                .unwrap();
                [&deck[slice_point..], &deck[..slice_point]].concat()
            }
            DealTechnique::Increment(inc) => {
                let mut index = BigInt::from(0);
                let mut next_deck = vec![BigInt::from(0); deck.len()];
                for card in deck {
                    next_deck[(index.clone() % BigInt::from(deck.len()))
                        .to_usize()
                        .unwrap()] = card.clone();
                    index += inc;
                }
                next_deck
            }
        }
    }

    fn undo_position(&self, position: &BigInt, deck_length: &BigInt) -> BigInt {
        match self {
            DealTechnique::NewStack => deck_length - 1 - position,
            DealTechnique::Cut(index) => {
                let slice_point = if *index > BigInt::from(0) {
                    index.clone()
                } else {
                    deck_length - index.abs()
                };
                let slice_point = deck_length - slice_point;
                if *position < slice_point {
                    deck_length - slice_point + position
                } else {
                    position - slice_point
                }
            }
            DealTechnique::Increment(inc) => {
                modular_inverse(&inc, &deck_length) * position % deck_length
            }
        }
    }
}

/// Extended euclidean algorithm described at: https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn modular_inverse(a: &BigInt, m: &BigInt) -> BigInt {
    let (r0, r1) = (m.clone(), a.clone());
    let (t0, t1) = (BigInt::from(0), BigInt::from(1));
    fn bezout_t(r0: &BigInt, r1: &BigInt, t0: &BigInt, t1: &BigInt) -> BigInt {
        if *r1 == BigInt::from(0) {
            return t0.clone();
        }
        let q = r0 / r1;
        let (r0, r1) = (r1, r0 - q.clone() * r1);
        let (t0, t1) = (t1, t0 - q.clone() * t1);

        bezout_t(&r0, &r1, &t0, &t1)
    }
    (bezout_t(&r0, &r1, &t0, &t1) + m) % m
}

fn bad_bad_bubble_sort(
    instructions: &Vec<DealTechnique>,
    deck_length: &BigInt,
) -> Vec<DealTechnique> {
    // Input is only like 100 elements long at most, this is fine

    // cut(a), inc(b) = inc(b), cut((a * b) % n)
    // cut(a), new() = new(), cut(-a)
    // inc(a), new() = new(), inc(a), cut(1 - a)

    let mut output = instructions.to_vec();
    'outer: loop {
        for i in 0..output.len() - 1 {
            match output[i].clone() {
                DealTechnique::Cut(a) => match output[i + 1].clone() {
                    DealTechnique::NewStack => {
                        output[i] = DealTechnique::NewStack;
                        output[i + 1] = DealTechnique::Cut(-a);

                        continue 'outer;
                    }
                    DealTechnique::Increment(b) => {
                        output[i] = DealTechnique::Increment(b.clone());
                        output[i + 1] = DealTechnique::Cut((a * b) % deck_length);

                        continue 'outer;
                    }
                    _ => (),
                },
                DealTechnique::Increment(a) => match output[i + 1].clone() {
                    DealTechnique::NewStack => {
                        output[i] = DealTechnique::NewStack;
                        output[i + 1] = DealTechnique::Increment(a.clone());
                        output.insert(i + 2, DealTechnique::Cut(BigInt::from(1) - a.clone()));

                        continue 'outer;
                    }
                    _ => (),
                },
                _ => (),
            };
        }
        break;
    }
    output
}

fn compress(instructions: &Vec<DealTechnique>, deck_length: &BigInt) -> Vec<DealTechnique> {
    // inc(a), inc(b) = inc((a * b) % n)
    // cut(a), cut(b) = cut((a + b) % n)
    // new(), new() = ()

    let mut output = bad_bad_bubble_sort(&instructions, deck_length);

    'outer: loop {
        for i in 0..output.len() - 1 {
            match &output[i] {
                DealTechnique::Increment(a) => match &output[i + 1] {
                    DealTechnique::Increment(b) => {
                        output[i] = DealTechnique::Increment((a * b) % deck_length);
                        output.remove(i + 1);
                        continue 'outer;
                    }
                    _ => (),
                },
                DealTechnique::Cut(a) => match &output[i + 1] {
                    DealTechnique::Cut(b) => {
                        output[i] = DealTechnique::Cut((a + b) % deck_length);
                        output.remove(i + 1);
                        continue 'outer;
                    }
                    _ => (),
                },
                DealTechnique::NewStack => match output[i + 1] {
                    DealTechnique::NewStack => {
                        output.remove(i);
                        output.remove(i);
                        continue 'outer;
                    }
                    _ => (),
                },
            }
        }
        break;
    }
    output
}

fn parse_input(filename: &str) -> Vec<DealTechnique> {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut instructions = vec![];
    for line in input.lines() {
        if line.starts_with("cut") {
            instructions.push(DealTechnique::Cut(line[4..].parse().unwrap()));
        } else if line.starts_with("deal into new stack") {
            instructions.push(DealTechnique::NewStack);
        } else if line.starts_with("deal with increment") {
            instructions.push(DealTechnique::Increment(line[20..].parse().unwrap()));
        }
    }
    instructions
}

fn main() {
    let instructions = parse_input("input");

    println!("22-1:");
    let mut deck = (0..=10_006)
        .map(|i| BigInt::from(i))
        .collect::<Vec<BigInt>>();
    for technique in &instructions {
        deck = technique.apply(&deck);
    }
    println!(
        "{}",
        deck.iter().position(|c| *c == BigInt::from(2019)).unwrap()
    );

    println!("22-2:");
    let iterations = BigInt::from(101_741_582_076_661 as u64);
    let deck_length = BigInt::from(119_315_717_514_047 as u64);
    let mut ending_card = BigInt::from(2020);

    let mut remaining_iterations = iterations.clone();
    while remaining_iterations > BigInt::from(0) {
        let mut compressed_instructions = compress(&instructions, &deck_length).to_vec();
        let mut iterations = BigInt::from(1);
        while &iterations * 2 < remaining_iterations {
            compressed_instructions.extend(compressed_instructions.to_vec());
            compressed_instructions = compress(&compressed_instructions, &deck_length);
            iterations *= 2;
        }
        for technique in compressed_instructions.iter().rev() {
            ending_card = technique.undo_position(&ending_card, &deck_length);
        }
        remaining_iterations -= iterations.clone();
    }
    println!("{}", ending_card);
}
