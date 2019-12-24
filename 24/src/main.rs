use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

fn parse_input(filename: &str) -> BugGrid {
    let input = std::fs::read_to_string(filename).unwrap();
    let mut tiles = HashMap::new();
    let width = input.lines().next().unwrap().trim().len();
    let height = input.lines().count();
    for (y, line) in input.lines().enumerate() {
        for (x, b) in line.chars().enumerate() {
            let bug = match b {
                '#' => true,
                '.' => false,
                _ => panic!("Unexpected input"),
            };
            tiles.insert((x as i32, y as i32), bug);
        }
    }
    BugGrid {
        tiles,
        width: width as i32,
        height: height as i32,
    }
}

#[derive(Debug, Clone)]
struct BugGrid {
    tiles: HashMap<(i32, i32), bool>,
    width: i32,
    height: i32,
}

impl BugGrid {
    fn tick(&self) -> BugGrid {
        let mut tiles = HashMap::new();

        for (x, y) in self.tiles.keys() {
            tiles.insert((*x, *y), self.next_tile(*x, *y));
        }

        BugGrid {
            tiles,
            width: self.width,
            height: self.height,
        }
    }

    fn next_tile(&self, x: i32, y: i32) -> bool {
        let mut adjacent_bugs = 0;
        for (cx, cy) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if *self.tiles.get(&(*cx, *cy)).unwrap_or(&false) {
                adjacent_bugs += 1;
            };
        }
        let tile = *self.tiles.get(&(x, y)).unwrap();
        if tile && adjacent_bugs == 1 {
            true
        } else if tile {
            false
        } else if adjacent_bugs == 1 || adjacent_bugs == 2 {
            true
        } else {
            false
        }
    }

    fn score(&self) -> u32 {
        let mut score = 0;

        for ((x, y), tile) in &self.tiles {
            if *tile {
                score |= 1 << (y * self.width + x);
            }
        }

        score
    }
}

impl std::fmt::Display for BugGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                output += match self.tiles.get(&(x, y)).unwrap() {
                    true => "#",
                    false => ".",
                };
            }
            output += "\n";
        }
        write!(f, "{}", output)
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum LayerAdjacency {
    Outer,
    Inner(Direction),
    None,
}

#[derive(Debug, Clone)]
struct RecursiveBugGrid {
    layers: HashMap<i32, HashMap<(i32, i32), bool>>,
    width: i32,
    height: i32,
}

impl RecursiveBugGrid {
    fn from(grid: &BugGrid) -> RecursiveBugGrid {
        let mut recursive_grid = RecursiveBugGrid {
            layers: HashMap::new(),
            width: grid.width,
            height: grid.height,
        };
        recursive_grid.layers.insert(0, grid.tiles.clone());
        recursive_grid.layers.insert(-1, recursive_grid.new_layer());
        recursive_grid.layers.insert(1, recursive_grid.new_layer());

        recursive_grid
    }

    fn new_layer(&self) -> HashMap<(i32, i32), bool> {
        let mut layer = HashMap::new();
        for y in 0..self.height {
            for x in 0..self.width {
                layer.insert((x, y), false);
            }
        }
        layer
    }

    fn adjacency(&self, x: i32, y: i32) -> LayerAdjacency {
        let center = self.center();

        if x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1 {
            LayerAdjacency::Outer
        } else if x == center.0 - 1 && y == center.1 {
            LayerAdjacency::Inner(Direction::Right)
        } else if x == center.0 + 1 && y == center.1 {
            LayerAdjacency::Inner(Direction::Left)
        } else if y == center.1 - 1 && x == center.0 {
            LayerAdjacency::Inner(Direction::Down)
        } else if y == center.1 + 1 && x == center.0 {
            LayerAdjacency::Inner(Direction::Up)
        } else {
            LayerAdjacency::None
        }
    }

    fn center(&self) -> (i32, i32) {
        (self.width / 2, self.height / 2)
    }

    fn tick(&self) -> RecursiveBugGrid {
        let mut layers = self.layers.clone();

        for (depth, layer) in &mut layers {
            for ((x, y), _) in layer.clone() {
                if self.center() == (x, y) {
                    continue;
                }
                layer.insert((x, y), self.next_tile(*depth, x, y));
            }
        }

        // Create new layers when bugs start to inhabit the end layers
        let (min_depth, max_depth) = layers
            .keys()
            .fold((0, 0), |(mi, ma), depth| (min(mi, *depth), max(ma, *depth)));
        if layers[&min_depth].values().any(|b| *b == true) {
            layers.entry(min_depth - 1).or_insert(self.new_layer());
        }
        if layers[&max_depth].values().any(|b| *b == true) {
            layers.entry(max_depth + 1).or_insert(self.new_layer());
        }

        RecursiveBugGrid {
            layers,
            width: self.width,
            height: self.height,
        }
    }

    fn count_tile(&self, depth: i32, from_x: i32, from_y: i32, x: i32, y: i32) -> i32 {
        let center = self.center();
        if x < 0 {
            self.count_tile(depth - 1, x, y, center.0 - 1, center.1)
        } else if x >= self.width {
            self.count_tile(depth - 1, x, y, center.0 + 1, center.1)
        } else if y < 0 {
            self.count_tile(depth - 1, x, y, center.0, center.1 - 1)
        } else if y >= self.height {
            self.count_tile(depth - 1, x, y, center.0, center.1 + 1)
        } else if center == (x, y) {
            // Count the entire adjacent row of an inner grid
            match self.adjacency(from_x, from_y) {
                LayerAdjacency::Inner(Direction::Up) => (0..self.width)
                    .zip(vec![self.height - 1; self.width as usize])
                    .map(|(x, y)| self.count_tile(depth + 1, center.0, center.1, x, y))
                    .sum::<i32>(),
                LayerAdjacency::Inner(Direction::Down) => (0..self.width)
                    .zip(vec![0; self.width as usize])
                    .map(|(x, y)| self.count_tile(depth + 1, center.0, center.1, x, y))
                    .sum::<i32>(),
                LayerAdjacency::Inner(Direction::Left) => (0..self.height)
                    .zip(vec![self.width - 1; self.height as usize])
                    .map(|(y, x)| self.count_tile(depth + 1, center.0, center.1, x, y))
                    .sum::<i32>(),
                LayerAdjacency::Inner(Direction::Right) => (0..self.height)
                    .zip(vec![0; self.height as usize])
                    .map(|(y, x)| self.count_tile(depth + 1, center.0, center.1, x, y))
                    .sum::<i32>(),
                _ => panic!("Should always be inner adjacency"),
            }
        } else {
            match self.layers.get(&depth) {
                Some(layer) => {
                    if *layer.get(&(x, y)).unwrap() {
                        1
                    } else {
                        0
                    }
                }
                None => 0,
            }
        }
    }

    fn next_tile(&self, depth: i32, x: i32, y: i32) -> bool {
        let mut adjacent_bugs = 0;
        for (cx, cy) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            adjacent_bugs += self.count_tile(depth, x, y, *cx, *cy);
        }
        let tile = *self.layers[&depth].get(&(x, y)).unwrap();
        if tile && adjacent_bugs == 1 {
            true
        } else if tile {
            false
        } else if adjacent_bugs == 1 || adjacent_bugs == 2 {
            true
        } else {
            false
        }
    }

    fn score(&self) -> u32 {
        let mut score = 0;

        for (_, layer) in &self.layers {
            for tile in layer.values() {
                if *tile {
                    score += 1;
                }
            }
        }

        score
    }
}

fn main() {
    let input = parse_input("input");

    println!("24-1:");
    let mut previous_scores = HashSet::<u32>::new();
    let mut grid = input.clone();
    loop {
        let score = grid.score();
        if previous_scores.contains(&score) {
            println!("{}", score);
            break;
        }
        previous_scores.insert(score);
        grid = grid.tick();
    }

    println!("24-2:");
    let mut grid = RecursiveBugGrid::from(&input);
    for _ in 0..200 {
        grid = grid.tick();
    }
    println!("{}", grid.score());
}
