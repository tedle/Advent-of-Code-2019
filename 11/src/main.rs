extern crate intcode;

use std::cmp::{max, min};

type Position = (i64, i64);

struct BoundingBox {
    min: Position,
    max: Position,
}

#[derive(Debug, PartialEq)]
enum HullColour {
    BLACK,
    WHITE,
}

#[derive(Debug, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    fn turn(&self, direction: Direction) -> Direction {
        match direction {
            Direction::LEFT if *self == Direction::UP => Direction::LEFT,
            Direction::LEFT if *self == Direction::LEFT => Direction::DOWN,
            Direction::LEFT if *self == Direction::DOWN => Direction::RIGHT,
            Direction::LEFT if *self == Direction::RIGHT => Direction::UP,
            Direction::RIGHT if *self == Direction::UP => Direction::RIGHT,
            Direction::RIGHT if *self == Direction::RIGHT => Direction::DOWN,
            Direction::RIGHT if *self == Direction::DOWN => Direction::LEFT,
            Direction::RIGHT if *self == Direction::LEFT => Direction::UP,
            _ => panic!("Unexpected direction output"),
        }
    }
}

struct EmergencyHullPaintingRobot {
    cpu: intcode::Cpu,
    painted_tiles: std::collections::HashMap<Position, HullColour>,
    direction: Direction,
    pos: Position,
}

impl EmergencyHullPaintingRobot {
    fn new(rom: Vec<i64>) -> EmergencyHullPaintingRobot {
        EmergencyHullPaintingRobot {
            cpu: intcode::Cpu::new(rom),
            painted_tiles: std::collections::HashMap::new(),
            direction: Direction::UP,
            pos: (0, 0),
        }
    }

    fn print_tiles(&self) {
        let mut bounds = BoundingBox {
            min: self.pos,
            max: self.pos,
        };
        for tile in &self.painted_tiles {
            if *tile.1 == HullColour::WHITE {
                bounds.min = (min(bounds.min.0, (tile.0).0), min(bounds.min.1, (tile.0).1));
                bounds.max = (max(bounds.max.0, (tile.0).0), max(bounds.max.1, (tile.0).1));
            }
        }
        // Ok, well, a nested loop would look way nicer than this monstrosity
        // But it's functional so that means this is actually amazing
        let tile_string = (bounds.min.1..=bounds.max.1)
            .flat_map(|y| {
                (bounds.min.0..=bounds.max.0)
                    .map(move |x| match self.painted_tiles.get(&(x, y)) {
                        Some(HullColour::WHITE) => '#',
                        _ => '.',
                    })
                    .chain(std::iter::once('\n'))
            })
            .collect::<String>();
        print!("{}", tile_string);
    }

    fn run(&mut self) {
        loop {
            let tile = self
                .painted_tiles
                .entry(self.pos)
                .or_insert(HullColour::BLACK);
            let input: i64 = match tile {
                HullColour::BLACK => 0,
                HullColour::WHITE => 1,
            };
            let colour = match self.cpu.run_with(&vec![input]) {
                Some(value) => value,
                None => return,
            };
            let direction = match self.cpu.run() {
                Some(value) => value,
                None => return,
            };
            *tile = match colour {
                0 => HullColour::BLACK,
                1 => HullColour::WHITE,
                _ => panic!("Unexpected colour output"),
            };
            self.direction = self.direction.turn(match direction {
                0 => Direction::LEFT,
                1 => Direction::RIGHT,
                _ => panic!("Unexpected direction output"),
            });
            self.pos = match self.direction {
                Direction::UP => (self.pos.0, self.pos.1 - 1),
                Direction::RIGHT => (self.pos.0 + 1, self.pos.1),
                Direction::LEFT => (self.pos.0 - 1, self.pos.1),
                Direction::DOWN => (self.pos.0, self.pos.1 + 1),
            };
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");
    println!("11-1:");
    let mut robot = EmergencyHullPaintingRobot::new(rom.to_vec());
    robot.run();
    println!("{}", robot.painted_tiles.len());
    println!("11-2:");
    robot = EmergencyHullPaintingRobot::new(rom.to_vec());
    robot.painted_tiles.insert(robot.pos, HullColour::WHITE);
    robot.run();
    robot.print_tiles();
}
