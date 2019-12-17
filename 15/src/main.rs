extern crate intcode;

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, VecDeque};

type Position = (i64, i64);

#[derive(Copy, Clone, Debug, PartialEq)]
enum TileKind {
    EMPTY,
    WALL,
    GOAL,
}

impl TileKind {
    fn from(i: i64) -> TileKind {
        match i {
            0 => TileKind::WALL,
            1 => TileKind::EMPTY,
            2 => TileKind::GOAL,
            _ => panic!("Invalid tile type"),
        }
    }
}

#[derive(Copy, Clone)]
struct Tile {
    kind: TileKind,
    cost: Option<i64>,
    parent: Option<Position>,
}

impl Tile {
    fn new(kind: TileKind, cost: Option<i64>, parent: Option<Position>) -> Tile {
        Tile { kind, cost, parent }
    }
}

type Map = HashMap<Position, Tile>;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Input {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Input {
    fn to_i64(&self) -> i64 {
        match self {
            Input::UP => 1,
            Input::DOWN => 2,
            Input::LEFT => 3,
            Input::RIGHT => 4,
        }
    }
}

struct LocatorRobot {
    cpu: intcode::Cpu,
    origin: Position,
    unsolved: BTreeMap<i64, VecDeque<Position>>,
    map: Map,
}

impl LocatorRobot {
    fn new(cpu: intcode::Cpu) -> LocatorRobot {
        let mut map = Map::new();
        let origin = (0, 0);
        map.insert(origin, Tile::new(TileKind::EMPTY, Some(0), None));
        LocatorRobot {
            cpu,
            origin,
            unsolved: BTreeMap::new(),
            map,
        }
    }

    fn move_from_origin_to(&self, pos: Position, robot: &mut intcode::Cpu) {
        fn move_to(map: &Map, pos: Position, to: Option<Position>) -> Vec<i64> {
            let mut inputs: Vec<i64> = vec![];
            let tile = map.get(&pos).unwrap();
            if let Some(from) = tile.parent {
                inputs.extend(move_to(map, from, Some(pos)));
            }
            if let Some(to) = to {
                if to.0 > pos.0 {
                    inputs.push(Input::RIGHT.to_i64());
                } else if to.0 < pos.0 {
                    inputs.push(Input::LEFT.to_i64());
                } else if to.1 > pos.1 {
                    inputs.push(Input::DOWN.to_i64());
                } else if to.1 < pos.1 {
                    inputs.push(Input::UP.to_i64());
                }
            }
            inputs
        }
        robot.push(&move_to(&self.map, pos, None));
        while robot.inputs.len() > 0 {
            robot.run();
        }
    }

    fn solve(&mut self) {
        self.solve_position(self.origin);
    }

    fn solve_position(&mut self, pos: Position) {
        let current_tile = self.map.get(&pos).unwrap();
        let current_tile_cost = current_tile.cost.expect("Cannot solve for a wall");
        for direction in [Input::UP, Input::DOWN, Input::LEFT, Input::RIGHT].iter() {
            let next_pos = match direction {
                Input::UP => (pos.0, pos.1 - 1),
                Input::DOWN => (pos.0, pos.1 + 1),
                Input::LEFT => (pos.0 - 1, pos.1),
                Input::RIGHT => (pos.0 + 1, pos.1),
            };
            let next_tile = if let Some(mut tile) = self.map.get_mut(&next_pos).cloned() {
                if tile.kind == TileKind::WALL {
                    continue;
                }
                if tile.cost.unwrap() > current_tile_cost + 1 {
                    tile.cost = Some(current_tile_cost + 1);
                    tile.parent = Some(pos);
                    self.add_unsolved(tile.cost.unwrap(), next_pos);
                }
                tile
            } else {
                // Could massively speed this up if we cached CPU states for each location
                // But it's still only a couple seconds run time, thanks native compilers
                let mut robot = self.cpu.clone();
                self.move_from_origin_to(pos, &mut robot);
                let kind = TileKind::from(robot.run_with(&vec![direction.to_i64()]).unwrap());
                let cost = match kind {
                    TileKind::WALL => None,
                    _ => Some(current_tile_cost + 1),
                };
                if let Some(cost) = cost {
                    self.add_unsolved(cost, next_pos);
                }
                Tile::new(kind, cost, Some(pos))
            };
            self.map.insert(next_pos, next_tile);
        }
        if let Some(next_pos) = self.pop_unsolved() {
            self.solve_position(next_pos);
        }
    }

    fn add_unsolved(&mut self, cost: i64, pos: Position) {
        self.unsolved
            .entry(cost)
            .and_modify(|tile_queue| {
                tile_queue.push_back(pos);
            })
            .or_insert_with(|| {
                let mut tile_queue: VecDeque<Position> = VecDeque::new();
                tile_queue.push_back(pos);
                tile_queue
            });
    }

    fn pop_unsolved(&mut self) -> Option<Position> {
        let mut pos = None;
        // Gets lowest cost queue since BTreeMap is sorted by key (cost)
        if let Some(key) = self.unsolved.keys().cloned().next() {
            let queue = self.unsolved.get_mut(&key).unwrap();
            pos = queue.pop_front();
            if queue.len() == 0 {
                self.unsolved.remove(&key);
            }
        }
        pos
    }

    #[allow(dead_code)]
    fn print_map(&self) {
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (0, 0, 0, 0);
        for (x, y) in self.map.keys() {
            min_x = min(min_x, *x);
            min_y = min(min_y, *y);
            max_x = max(max_x, *x);
            max_y = max(max_y, *y);
        }
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let tile = self.map.get(&(x, y));
                let tile_display = match tile {
                    Some(t) => match t.kind {
                        TileKind::EMPTY => ' ',
                        TileKind::WALL => '#',
                        TileKind::GOAL => '!',
                    },
                    None => '~',
                };
                print!("{}", tile_display);
            }
            println!("");
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");
    println!("15-1:");
    let mut robot = LocatorRobot::new(intcode::Cpu::new(rom.to_vec()));
    robot.solve();
    let (pos, goal) = robot
        .map
        .iter()
        .filter(|(_, t)| t.kind == TileKind::GOAL)
        .next()
        .unwrap();
    println!("{}", goal.cost.unwrap());
    println!("15-2:");
    let mut cpu = intcode::Cpu::new(rom.to_vec());
    // Re-using solved completion state to a set base CPU state where the origin is the goal
    robot.move_from_origin_to(*pos, &mut cpu);
    let mut robot = LocatorRobot::new(cpu);
    robot.solve();
    let time = robot
        .map
        .values()
        .fold(0, |acc, tile| max(acc, tile.cost.unwrap_or(0)));
    println!("{}", time);
}
