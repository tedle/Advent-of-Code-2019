use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

type Position = (i64, i64);

#[derive(Copy, Clone, Debug, PartialEq)]
enum TileKind {
    EMPTY,
    WALL,
    ORIGIN,
    KEY(char),
    DOOR(char),
}

impl TileKind {
    fn from(c: char) -> TileKind {
        match c {
            '#' => TileKind::WALL,
            '.' => TileKind::EMPTY,
            '@' => TileKind::ORIGIN,
            'a'..='z' => TileKind::KEY(c),
            'A'..='Z' => TileKind::DOOR(c.to_ascii_lowercase()),
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
    unsolved: OrderedQueue<Position>,
    map: Map,
}

impl MazeSolver {
    fn new(map: &Map, origin: Position) -> MazeSolver {
        let mut map = map.clone();
        map.insert(origin, Tile::new(TileKind::EMPTY, Some(0), None));
        MazeSolver {
            origin,
            unsolved: OrderedQueue::new(),
            map: map.clone(),
        }
    }

    fn solve(&mut self, ignore_doors: bool) {
        self.solve_position(self.origin, ignore_doors);
    }

    fn solve_position(&mut self, pos: Position, ignore_doors: bool) {
        let current_tile = self.map.get(&pos).unwrap();
        let current_tile_cost = current_tile.cost.expect("Cannot solve for a wall");
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
                        TileKind::DOOR(_) if ignore_doors == false => continue,
                        _ => {}
                    }
                    if tile.cost.is_none() || tile.cost.unwrap() > current_tile_cost + 1 {
                        tile.cost = Some(current_tile_cost + 1);
                        tile.parent = Some(pos);
                        self.unsolved.add(tile.cost.unwrap(), next_pos);
                    }
                    tile
                }
                None => panic!("Reached out of map bounds somehow"),
            };
            self.map.insert(*next_pos, next_tile);
        }
        if let Some(next_pos) = self.unsolved.pop() {
            self.solve_position(next_pos.1, ignore_doors);
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
                        TileKind::ORIGIN => '@',
                        TileKind::KEY(c) => c,
                        TileKind::DOOR(c) => c.to_ascii_uppercase(),
                    },
                    None => '~',
                };
                print!("{}", tile_display);
            }
            println!("");
        }
    }
}

fn parse_input(filename: &str) -> (Map, Vec<Position>) {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut map = Map::new();
    let mut origins = vec![];
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let kind = TileKind::from(c);
            let pos = (x as i64, y as i64);
            map.insert(pos, Tile::new(kind, None, None));
            if kind == TileKind::ORIGIN {
                origins.push(pos);
            }
        }
    }
    (map, origins)
}

#[derive(Debug, Clone)]
struct Key {
    distances: HashMap<char, i64>,
    distance_to_origin: i64,
    requirements: HashSet<char>,
}

impl Key {
    fn can_unlock(&self, found_keys: &HashSet<char>) -> bool {
        self.requirements.is_subset(found_keys)
    }
}

struct KeySolver {
    keys: Vec<HashMap<char, Key>>,
    cache: HashMap<(char, i32), i64>,
}

impl KeySolver {
    fn new(map: &Map, origins: &Vec<Position>) -> KeySolver {
        let map = map.clone();
        let keys = KeySolver::generate_keys(&map, origins);

        KeySolver {
            keys,
            cache: HashMap::new(),
        }
    }

    fn generate_keys(map: &Map, origins: &Vec<Position>) -> Vec<HashMap<char, Key>> {
        let mut keyring = vec![];
        for origin in origins {
            let mut solved_keys: HashMap<char, Key> = HashMap::new();
            let mut origin_map = MazeSolver::new(&map, *origin);
            origin_map.solve(true);
            let keys: Vec<(Position, char, i64)> = origin_map
                .map
                .iter()
                .filter_map(|t| match t.1.kind {
                    TileKind::KEY(c) if t.1.cost.is_some() => Some((*t.0, c, t.1.cost.unwrap())),
                    _ => None,
                })
                .collect();
            for key in &keys {
                let mut solved_map = MazeSolver::new(&map, key.0);
                solved_map.solve(true);
                let distances: HashMap<char, i64> = solved_map
                    .map
                    .iter()
                    .filter_map(|t| match t.1.kind {
                        TileKind::KEY(c) if c != key.1 && t.1.cost.is_some() => {
                            Some((c, t.1.cost.unwrap()))
                        }
                        _ => None,
                    })
                    .collect();

                let mut current_pos = *origin;
                let mut requirements: HashSet<char> = HashSet::new();
                loop {
                    let tile = solved_map.map.get(&current_pos).unwrap();
                    match tile.kind {
                        TileKind::DOOR(c) => {
                            requirements.insert(c);
                        }
                        _ => {}
                    }
                    if let Some(pos) = tile.parent {
                        current_pos = pos;
                    } else {
                        break;
                    }
                }
                solved_keys.insert(
                    key.1,
                    Key {
                        distances,
                        requirements,
                        distance_to_origin: key.2,
                    },
                );
            }
            keyring.push(solved_keys);
        }

        let other_keyring = keyring.clone();
        for (i, keys) in keyring.iter_mut().enumerate() {
            for (_, key) in keys {
                fn find_requirements(
                    requirements: &HashSet<char>,
                    keyring: &Vec<HashMap<char, Key>>,
                ) -> HashSet<char> {
                    let mut full_requirements = requirements.clone();
                    for requirement in requirements {
                        for keys in keyring {
                            if let Some(key) = keys.get(&requirement) {
                                full_requirements
                                    .extend(find_requirements(&key.requirements, keyring));
                            }
                        }
                    }
                    full_requirements
                }
                let full_requirements = find_requirements(&key.requirements, &other_keyring);
                let owned_keys = &other_keyring[i].keys().cloned().collect::<HashSet<char>>();
                key.requirements = owned_keys
                    .intersection(&full_requirements)
                    .cloned()
                    .collect::<HashSet<char>>();
            }
        }
        keyring
    }

    fn keyset_as_bitfield(set: &HashSet<char>) -> i32 {
        let mut field = 0;
        for c in set {
            field |= 1 << (*c as u8 - 'a' as u8);
        }
        field
    }

    fn find_keys(&mut self) -> i64 {
        fn find(
            solver: &mut KeySolver,
            keys: &HashMap<char, Key>,
            found_keys: HashSet<char>,
            cost: i64,
            from: Option<char>,
        ) -> i64 {
            let mut best: Option<i64> = None;
            for (name, key) in keys
                .iter()
                .filter(|(c, k)| !found_keys.contains(c) && k.can_unlock(&found_keys))
            {
                let mut next_found_keys = found_keys.clone();
                next_found_keys.insert(*name);
                let next_cost = match from {
                    Some(c) => *key.distances.get(&c).unwrap(),
                    None => key.distance_to_origin,
                };
                let cache_key = (*name, KeySolver::keyset_as_bitfield(&next_found_keys));
                let found_cost = match solver.cache.get(&cache_key).clone() {
                    Some(c) => *c + cost + next_cost,
                    None => {
                        let c = find(solver, keys, next_found_keys, cost + next_cost, Some(*name));
                        solver.cache.insert(cache_key, c - cost - next_cost);
                        c
                    }
                };
                best = match best {
                    Some(c) => Some(min(c, found_cost)),
                    None => Some(found_cost),
                };
            }
            best.unwrap_or(cost)
        }
        let keys = self.keys.clone();
        keys.iter()
            .map(|key| find(self, key, HashSet::new(), 0, None))
            .sum()
    }
}

fn main() {
    let (map, origins) = parse_input("input");
    let mut solver = KeySolver::new(&map, &origins);
    println!("18-1:\n{}", solver.find_keys());
    let (map, origins) = parse_input("input2");
    let mut solver = KeySolver::new(&map, &origins);
    println!("18-2:\n{}", solver.find_keys());
}
