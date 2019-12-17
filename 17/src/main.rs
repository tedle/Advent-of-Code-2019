extern crate intcode;
use std::collections::HashMap;

type Position = (i64, i64);

#[derive(Copy, Clone, Debug, PartialEq)]
enum TileKind {
    EMPTY,
    FLOOR,
}

type Map = HashMap<Position, TileKind>;

fn scan_map(rom: &Vec<i64>) -> Map {
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    let mut pos: Position = (0, 0);
    let mut map = Map::new();
    loop {
        let mut newline = false;
        match cpu.run() {
            Some(c) => match c as u8 as char {
                '#' | '^' | 'v' | '<' | '>' => {
                    map.insert(pos, TileKind::FLOOR);
                }
                '.' | 'X' => {
                    map.insert(pos, TileKind::EMPTY);
                }
                '\n' => {
                    newline = true;
                }
                _ => {}
            },
            None => break,
        }
        match newline {
            true => pos = (0, pos.1 + 1),
            false => pos.0 += 1,
        }
    }
    map
}

fn intersection_score(map: &Map) -> i64 {
    let mut score = 0;
    let tiles = map.iter().filter(|t| *t.1 == TileKind::FLOOR);
    for (pos, _) in tiles {
        if let Some(TileKind::EMPTY) | None = map.get(&(pos.0 - 1, pos.1)) {
            continue;
        }
        if let Some(TileKind::EMPTY) | None = map.get(&(pos.0 + 1, pos.1)) {
            continue;
        }
        if let Some(TileKind::EMPTY) | None = map.get(&(pos.0, pos.1 - 1)) {
            continue;
        }
        if let Some(TileKind::EMPTY) | None = map.get(&(pos.0, pos.1 + 1)) {
            continue;
        }
        score += pos.0 * pos.1;
    }
    score
}

fn run(rom: &Vec<i64>, path_input: &str, print: bool) -> Option<i64> {
    let mut last_output = None;
    let mut rom = rom.to_vec();
    rom[0] = 2;
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    let input = path_input.bytes().map(|b| b as i64).collect::<Vec<i64>>();
    cpu.push(&input);
    loop {
        match cpu.run() {
            Some(c) => last_output = Some(c),
            None => return last_output,
        }
        if print {
            print!("{}", last_output.unwrap() as u8 as char);
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");
    println!("17-1:");
    let map = scan_map(&rom);
    println!("{}", intersection_score(&map));
    println!("17-2:");
    // Probably wasn't intended to solve this by hand, but it was a fun and easy not-so-coding-related puzzle.
    let output = run(
        &rom,
        "A,B,A,B,C,A,C,A,C,B\n\
        R,12,L,8,L,4,L,4\n\
        L,8,R,6,L,6\n\
        L,8,L,4,R,12,L,6,L,4\n\
        n\n",
        false,
    )
    .unwrap();
    println!("{}", output);
}
