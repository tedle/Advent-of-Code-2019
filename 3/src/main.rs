use std::cmp;
use std::fs;

#[derive(Debug)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn distance_from_origin(self: &Point) -> i64 {
        self.x.abs() + self.y.abs()
    }
}

type Layout = Vec<(Point, Point, i64)>;
type IntersectionList = Vec<(Point, i64)>;

fn parse_layout(input: &str) -> Layout {
    let paths: Vec<(Direction, i64)> = input
        .trim()
        .split(",")
        .map(|path| {
            let direction = match &path[0..1] {
                "U" => Direction::UP,
                "D" => Direction::DOWN,
                "L" => Direction::LEFT,
                "R" => Direction::RIGHT,
                _ => panic!(),
            };
            let steps: i64 = path[1..].parse().unwrap();
            (direction, steps)
        })
        .collect();

    let mut wires: Layout = vec![];
    let mut pos = Point { x: 0, y: 0 };
    let mut total_distance = 0;
    for path in paths {
        let next_pos = match path.0 {
            Direction::UP => Point {
                y: pos.y + path.1,
                ..pos
            },
            Direction::DOWN => Point {
                y: pos.y - path.1,
                ..pos
            },
            Direction::LEFT => Point {
                x: pos.x - path.1,
                ..pos
            },
            Direction::RIGHT => Point {
                x: pos.x + path.1,
                ..pos
            },
        };
        wires.push((pos, next_pos, total_distance));
        total_distance += (pos.x - next_pos.x).abs() + (pos.y - next_pos.y).abs();
        pos = next_pos;
    }

    wires
}

fn find_intersections(a: &Layout, b: &Layout) -> IntersectionList {
    let a_vertical_wires: Layout = a.iter().filter(|p| p.0.x == p.1.x).cloned().collect();
    let a_horizontal_wires: Layout = a.iter().filter(|p| p.0.y == p.1.y).cloned().collect();
    let b_vertical_wires: Layout = b.iter().filter(|p| p.0.x == p.1.x).cloned().collect();
    let b_horizontal_wires: Layout = b.iter().filter(|p| p.0.y == p.1.y).cloned().collect();

    let mut intersections: IntersectionList = vec![];

    let mut search = |vertical_wires: Layout, horizontal_wires: Layout| {
        for v_wire in &vertical_wires {
            for h_wire in &horizontal_wires {
                if v_wire.0.x > cmp::min(h_wire.0.x, h_wire.1.x)
                    && v_wire.0.x < cmp::max(h_wire.0.x, h_wire.1.x)
                    && h_wire.0.y > cmp::min(v_wire.0.y, v_wire.1.y)
                    && h_wire.0.y < cmp::max(v_wire.0.y, v_wire.1.y)
                {
                    let v_wire_extra_distance = (v_wire.0.y - h_wire.0.y).abs();
                    let h_wire_extra_distance = (h_wire.0.x - v_wire.0.x).abs();
                    intersections.push((
                        Point {
                            x: v_wire.0.x,
                            y: h_wire.0.y,
                        },
                        v_wire.2 + h_wire.2 + v_wire_extra_distance + h_wire_extra_distance,
                    ));
                }
            }
        }
    };
    search(a_vertical_wires, b_horizontal_wires);
    search(b_vertical_wires, a_horizontal_wires);

    intersections
}

fn main() {
    let input = fs::read_to_string("input").unwrap();
    let layouts: Vec<Layout> = input.lines().map(|line| parse_layout(line)).collect();

    assert_eq!(layouts.len(), 2);

    let intersections = find_intersections(&layouts[0], &layouts[1]);

    let mut distances_from_origin: Vec<i64> = intersections
        .iter()
        .map(|p| p.0.distance_from_origin())
        .collect();
    distances_from_origin.sort();
    println!("3-1:\n{:?}", distances_from_origin[0]);
    let mut total_distances: Vec<i64> = intersections.iter().map(|p| p.1).collect();
    total_distances.sort();
    println!("3-2:\n{:?}", total_distances[0]);
}
