use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, VecDeque};

type Position = (i64, i64);

#[derive(Copy, Clone, Debug, PartialEq)]
enum TileKind {
    EMPTY,
    WALL,
    ENTRANCE,
    EXIT,
    WARP((char, char), bool),
}

#[derive(Debug, Copy, Clone)]
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

struct OrderedQueue<T> {
    queue: BTreeMap<i64, VecDeque<T>>,
}

impl<T> OrderedQueue<T>
where
    T: Clone,
{
    fn new() -> OrderedQueue<T> {
        OrderedQueue::<T> {
            queue: BTreeMap::new(),
        }
    }

    fn add(&mut self, cost: i64, value: &T) {
        self.queue
            .entry(cost)
            .and_modify(|queue| {
                queue.push_back(value.clone());
            })
            .or_insert_with(|| {
                let mut queue: VecDeque<T> = VecDeque::new();
                queue.push_back(value.clone());
                queue
            });
    }

    fn pop(&mut self) -> Option<(i64, T)> {
        let mut value = None;
        // Gets lowest cost queue since BTreeMap is sorted by key (cost)
        if let Some(key) = self.queue.keys().cloned().next() {
            let queue = self.queue.get_mut(&key).unwrap();
            value = Some((key, queue.pop_front().unwrap()));
            if queue.len() == 0 {
                self.queue.remove(&key);
            }
        }
        value
    }
}

struct MazeSolver {
    origin: Position,
    unsolved: OrderedQueue<(Position, usize)>,
    map: Map,
    inner_maps: Vec<Map>,
    inner_exit_cost: Option<i64>,
}

impl MazeSolver {
    fn new(map: &Map, origin: Position) -> MazeSolver {
        let mut map = map.clone();
        map.insert(origin, Tile::new(TileKind::ENTRANCE, Some(0), None));
        MazeSolver {
            origin,
            unsolved: OrderedQueue::new(),
            map: map,
            inner_maps: vec![],
            inner_exit_cost: None,
        }
    }

    fn solve(&mut self) {
        self.solve_position(self.origin);
    }

    fn solve_position(&mut self, pos: Position) {
        let current_tile = self.map.get(&pos).unwrap();
        let current_tile_cost = current_tile.cost.expect("Cannot solve for a wall");

        if let TileKind::WARP(id, inner) = current_tile.kind {
            let (exit_pos, mut exit_tile) = self
                .map
                .iter_mut()
                .filter(|(_, t)| t.kind == TileKind::WARP(id, !inner))
                .next()
                .unwrap();
            if exit_tile.cost.is_none() || exit_tile.cost.unwrap() > current_tile_cost + 1 {
                exit_tile.cost = Some(current_tile_cost + 1);
                exit_tile.parent = Some(pos);
                self.unsolved.add(exit_tile.cost.unwrap(), &(*exit_pos, 0));
            }
        };

        for next_pos in [
            (pos.0, pos.1 - 1),
            (pos.0, pos.1 + 1),
            (pos.0 - 1, pos.1),
            (pos.0 + 1, pos.1),
        ]
        .iter()
        {
            let next_tile = match self.map.get_mut(&next_pos).cloned() {
                Some(mut tile) => {
                    match tile.kind {
                        TileKind::WALL => continue,
                        _ => {}
                    }
                    if tile.cost.is_none() || tile.cost.unwrap() > current_tile_cost + 1 {
                        tile.cost = Some(current_tile_cost + 1);
                        tile.parent = Some(pos);
                        self.unsolved.add(tile.cost.unwrap(), &(*next_pos, 0));
                    }
                    tile
                }
                None => panic!("Reached out of map bounds somehow"),
            };
            self.map.insert(*next_pos, next_tile);
        }
        if let Some((_, next)) = self.unsolved.pop() {
            self.solve_position(next.0);
        }
    }

    fn new_layer(&self) -> Map {
        let mut next_map = self.map.clone();
        for (_, tile) in &mut next_map {
            tile.cost = None;
            tile.parent = None;
        }
        next_map
    }

    /// Since a recursive maze can go on forever, this will terminate early once an exit is found leaving an inexhaustive map state.
    fn solve_recursive(&mut self) -> i64 {
        self.inner_exit_cost = None;
        if self.inner_maps.len() == 0 {
            let mut next_map = self.new_layer();
            next_map.insert(self.origin, Tile::new(TileKind::ENTRANCE, Some(0), None));
            self.inner_maps.push(next_map);
        }
        self.solve_position_recursive(self.origin, 0);
        // Ironically the recursive maze needs a non-recursive solution (stack overflow)
        loop {
            if let Some(cost) = self.inner_exit_cost {
                return cost;
            }
            if let Some((_, next)) = self.unsolved.pop() {
                self.solve_position_recursive(next.0, next.1);
            } else {
                break;
            }
        }
        panic!("Could not find exit");
    }

    fn solve_position_recursive(&mut self, pos: Position, depth: usize) {
        while self.inner_maps.len() <= depth + 1 {
            self.inner_maps.push(self.new_layer());
        }

        let current_tile = self.inner_maps[depth].get(&pos).unwrap();
        let current_tile_cost = current_tile.cost.expect("Cannot solve for a wall");
        if current_tile.kind == TileKind::EXIT && depth == 0 {
            self.inner_exit_cost = Some(current_tile_cost);
            return;
        }

        if let TileKind::WARP(id, inner) = current_tile.kind {
            if inner == true || depth > 0 {
                let next_depth = if inner { depth + 1 } else { depth - 1 };
                let (exit_pos, mut exit_tile) = self.inner_maps[next_depth]
                    .iter_mut()
                    .filter(|(_, t)| t.kind == TileKind::WARP(id, !inner))
                    .next()
                    .unwrap();
                if exit_tile.cost.is_none() || exit_tile.cost.unwrap() > current_tile_cost + 1 {
                    exit_tile.cost = Some(current_tile_cost + 1);
                    exit_tile.parent = Some(pos);
                    self.unsolved
                        .add(exit_tile.cost.unwrap(), &(*exit_pos, next_depth));
                }
            }
        };

        for next_pos in [
            (pos.0, pos.1 - 1),
            (pos.0, pos.1 + 1),
            (pos.0 - 1, pos.1),
            (pos.0 + 1, pos.1),
        ]
        .iter()
        {
            let next_tile = match self.inner_maps[depth].get_mut(&next_pos).cloned() {
                Some(mut tile) => {
                    match tile.kind {
                        TileKind::WALL => continue,
                        _ => {}
                    }
                    if tile.cost.is_none() || tile.cost.unwrap() > current_tile_cost + 1 {
                        tile.cost = Some(current_tile_cost + 1);
                        tile.parent = Some(pos);
                        self.unsolved.add(tile.cost.unwrap(), &(*next_pos, depth));
                    }
                    tile
                }
                None => panic!("Reached out of map bounds somehow"),
            };
            self.inner_maps[depth].insert(*next_pos, next_tile);
        }
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
                        TileKind::ENTRANCE => '^',
                        TileKind::EXIT => 'v',
                        TileKind::WARP(_, true) => 'I',
                        TileKind::WARP(_, false) => 'O',
                    },
                    None => '~',
                };
                print!("{}", tile_display);
            }
            println!("");
        }
    }
}

fn try_tile(map: &Vec<Vec<char>>, x: i64, y: i64) -> Option<&char> {
    if x < 0 || y < 0 {
        return None;
    }
    match map.get(y as usize) {
        Some(row) => row.get(x as usize),
        None => None,
    }
}

fn parse_input(filename: &str) -> (Map, Position) {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut map = Map::new();
    let input = input
        .lines()
        .map(|line| line.chars().collect::<Vec<char>>())
        .collect::<Vec<_>>();
    let center = (input[0].len() as i64 / 2, input.len() as i64 / 2);
    for (y, row) in input.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let (x, y) = (x as i64, y as i64);
            let kind = match c {
                '.' => TileKind::EMPTY,
                '#' | _ => TileKind::WALL,
            };
            let pos = (x, y);
            map.entry(pos).or_insert(Tile::new(kind, None, None));
            if *c >= 'A' && *c <= 'Z' {
                if let Some(c2) = try_tile(&input, x + 1, y) {
                    if *c2 >= 'A' && *c2 <= 'Z' {
                        let pos = match try_tile(&input, x - 1, y) {
                            Some('.') => (x - 1, y),
                            _ => match try_tile(&input, x + 2, y) {
                                Some('.') => (x + 2, y),
                                _ => panic!("Unexpected portal layout"),
                            },
                        };
                        let kind = if *c == 'A' && *c2 == 'A' {
                            TileKind::ENTRANCE
                        } else if *c == 'Z' && *c2 == 'Z' {
                            TileKind::EXIT
                        } else {
                            let inner = if pos.0 < center.0 {
                                pos.0 < x
                            } else {
                                pos.0 > x
                            };
                            TileKind::WARP((*c, *c2), inner)
                        };
                        map.insert(pos, Tile::new(kind, None, None));
                    }
                }
                if let Some(c2) = try_tile(&input, x, y + 1) {
                    if *c2 >= 'A' && *c2 <= 'Z' {
                        let pos = match try_tile(&input, x, y - 1) {
                            Some('.') => (x, y - 1),
                            _ => match try_tile(&input, x, y + 2) {
                                Some('.') => (x, y + 2),
                                _ => panic!("Unexpected portal layout"),
                            },
                        };
                        let kind = if *c == 'A' && *c2 == 'A' {
                            TileKind::ENTRANCE
                        } else if *c == 'Z' && *c2 == 'Z' {
                            TileKind::EXIT
                        } else {
                            let inner = if pos.1 < center.1 {
                                pos.1 < y
                            } else {
                                pos.1 > y
                            };
                            TileKind::WARP((*c, *c2), inner)
                        };
                        map.insert(pos, Tile::new(kind, None, None));
                    }
                }
            }
        }
    }
    let origin = *map
        .iter()
        .filter(|(_, v)| v.kind == TileKind::ENTRANCE)
        .next()
        .expect("Could not find entrance")
        .0;
    (map, origin)
}

fn main() {
    let (map, origin) = parse_input("input");
    let mut solver = MazeSolver::new(&map, origin);
    solver.solve();
    println!(
        "20-1:\n{}",
        solver
            .map
            .values()
            .filter(|t| t.kind == TileKind::EXIT)
            .next()
            .unwrap()
            .cost
            .unwrap()
    );
    let (map, origin) = parse_input("input");
    let mut solver = MazeSolver::new(&map, origin);
    println!("20-2:\n{}", solver.solve_recursive());
}
