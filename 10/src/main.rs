#[derive(Copy, Clone, Debug)]
struct Position {
    x: i64,
    y: i64,
}
#[derive(Copy, Clone, Debug)]
struct Asteroid {
    pos: Position,
    visible: usize,
}
type AsteroidField = Vec<Vec<bool>>;

fn greatest_common_factor(a: i64, b: i64) -> i64 {
    let min = std::cmp::min(a.abs(), b.abs());
    if min == 0 {
        return std::cmp::max(a.abs(), b.abs());
    }

    for i in (1..=min).rev() {
        if a % i == 0 && b % i == 0 {
            return i;
        }
    }
    panic!("Could not find greatest common factor")
}

fn parse_asteroid_field(filename: &str) -> AsteroidField {
    let mut field: AsteroidField = vec![];
    let data = std::fs::read_to_string(filename).unwrap();
    let mut line: Vec<bool> = vec![];
    for c in data.chars() {
        match c {
            '#' => line.push(true),
            '.' => line.push(false),
            '\n' => {
                field.push(line.to_vec());
                line.clear();
            }
            _ => panic!("Unknown input data"),
        }
    }
    field
}

fn los(origin: Position, pos: Position, field: &AsteroidField) -> bool {
    let (dx, dy) = (pos.x - origin.x, pos.y - origin.y);
    if dx == 0 && dy == 0 {
        return false;
    }
    let factor = greatest_common_factor(dx, dy);
    let (x_step, y_step) = (dx / factor, dy / factor);
    let (mut x_trace, mut y_trace) = (origin.x + x_step, origin.y + y_step);
    while (dx, dy) != (x_trace - origin.x, y_trace - origin.y) {
        if field[y_trace as usize][x_trace as usize] {
            return false;
        }
        x_trace += x_step;
        y_trace += y_step;
    }
    true
}

fn rot(origin: Position, pos: Position) -> f64 {
    let (dx, dy) = (
        pos.x as f64 - origin.x as f64,
        pos.y as f64 - origin.y as f64,
    );
    let rot = dy.atan2(dx) + std::f64::consts::PI / 2.0;
    if rot < 0.0 {
        rot + std::f64::consts::PI * 2.0
    } else {
        rot
    }
}

fn count_visible_asteroids(x: i64, y: i64, field: &AsteroidField) -> usize {
    let mut count = 0;
    for (search_y, line) in field.iter().enumerate() {
        for (search_x, is_asteroid) in line.iter().enumerate() {
            if (search_x as i64 == x && search_y as i64 == y) || !*is_asteroid {
                continue;
            }
            if los(
                Position { x, y },
                Position {
                    x: search_x as i64,
                    y: search_y as i64,
                },
                field,
            ) {
                count += 1;
            }
        }
    }
    count
}

fn find_best_asteroid(field: &AsteroidField) -> Option<Asteroid> {
    let mut best: Option<Asteroid> = None;
    for (y, line) in field.iter().enumerate() {
        for (x, is_asteroid) in line.iter().enumerate() {
            if *is_asteroid {
                let visible = count_visible_asteroids(x as i64, y as i64, field);
                if best.is_none() || visible > best.unwrap().visible {
                    best = Some(Asteroid {
                        pos: Position {
                            x: x as i64,
                            y: y as i64,
                        },
                        visible,
                    });
                }
            }
        }
    }
    best
}

fn asteroid_destruction_queue(origin: Position, field: &AsteroidField) -> Vec<Asteroid> {
    let mut destruction_queue: Vec<Asteroid> = vec![];
    let mut next_field = field.to_vec();
    loop {
        let mut asteroids: Vec<Asteroid> = vec![];
        let current_field = next_field.to_vec();
        for (y, line) in current_field.iter().enumerate() {
            for (x, is_asteroid) in line.iter().enumerate() {
                if *is_asteroid {
                    asteroids.push(Asteroid {
                        pos: Position {
                            x: x as i64,
                            y: y as i64,
                        },
                        visible: 0,
                    });
                }
            }
        }
        // Will not destroy origin base, should always have 1 left over
        if asteroids.len() <= 1 {
            break;
        }
        asteroids = asteroids
            .into_iter()
            .filter(|a| los(origin, a.pos, &current_field))
            .collect::<Vec<_>>();
        asteroids.sort_by(|a, b| rot(origin, a.pos).partial_cmp(&rot(origin, b.pos)).unwrap());
        for asteroid in &asteroids {
            destruction_queue.push(*asteroid);
            next_field[asteroid.pos.y as usize][asteroid.pos.x as usize] = false;
        }
    }
    destruction_queue
}

fn main() {
    println!("10-1:");
    let field = parse_asteroid_field("input");
    let asteroid = find_best_asteroid(&field).unwrap();
    println!("{}", asteroid.visible);
    let winning_asteroid = asteroid_destruction_queue(asteroid.pos, &field)[199];
    println!(
        "10-2:\n{}",
        winning_asteroid.pos.x * 100 + winning_asteroid.pos.y
    );
}
