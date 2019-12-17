extern crate intcode;

use std::cmp::{max, min};
use std::collections::HashSet;

type Position = (i64, i64);

#[derive(Copy, Clone, Debug, PartialEq)]
enum Tile {
    EMPTY,
    WALL,
    BLOCK,
    HORIPAD,
    BALL,
}

impl Tile {
    fn from(i: i64) -> Tile {
        match i {
            0 => Tile::EMPTY,
            1 => Tile::WALL,
            2 => Tile::BLOCK,
            3 => Tile::HORIPAD,
            4 => Tile::BALL,
            _ => panic!("Invalid tile type"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Input {
    NONE,
    LEFT,
    RIGHT,
}

impl Input {
    fn to_i64(&self) -> i64 {
        match self {
            Input::NONE => 0,
            Input::LEFT => -1,
            Input::RIGHT => 1,
        }
    }
}

type Screen = std::collections::HashMap<Position, Tile>;

#[allow(dead_code)]
fn print_screen(screen: &Screen, score: i64) {
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0, 0, 0, 0);
    for (x, y) in screen.keys() {
        min_x = min(min_x, *x);
        min_y = min(min_y, *y);
        max_x = max(max_x, *x);
        max_y = max(max_y, *y);
    }
    println!("Score: {}", score);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let tile = screen.get(&(x, y)).unwrap_or(&Tile::EMPTY);
            let tile_display = match tile {
                Tile::EMPTY => ' ',
                Tile::WALL => '#',
                Tile::BLOCK => 'U',
                Tile::HORIPAD => '_',
                Tile::BALL => '@',
            };
            print!("{}", tile_display);
        }
        println!("");
    }
}

struct BallPredictionEngine {
    ball_momentum: Input,
    prev_ball_pos: (i64, i64),
    prev_screen: Screen,
}

impl BallPredictionEngine {
    fn new() -> BallPredictionEngine {
        BallPredictionEngine {
            ball_momentum: Input::RIGHT,
            prev_ball_pos: (0, 0),
            prev_screen: Screen::new(),
        }
    }

    fn move_paddle(&mut self, game: &mut intcode::Cpu, screen: &Screen) {
        // Do you like state machines and if/else branches? Welcome!
        let ball_pos = screen
            .iter()
            .filter(|e| *e.1 == Tile::BALL)
            .map(|e| e.0)
            .next();
        let paddle_pos = screen
            .iter()
            .filter(|e| *e.1 == Tile::HORIPAD)
            .map(|e| e.0)
            .next();
        let blocks = screen
            .iter()
            .filter(|e| *e.1 == Tile::BLOCK)
            .map(|e| *e.0)
            .collect::<HashSet<(i64, i64)>>();
        let prev_blocks = self
            .prev_screen
            .iter()
            .filter(|e| *e.1 == Tile::BLOCK)
            .map(|e| *e.0)
            .collect::<HashSet<(i64, i64)>>();
        // Block was broken? Check if we need to reverse ball momentum
        if blocks.len() < prev_blocks.len() {
            let ball_pos = ball_pos.unwrap();
            let block_pos = prev_blocks.difference(&blocks).next().unwrap();
            self.ball_momentum = if ball_pos.0 < block_pos.0 {
                Input::LEFT
            } else if ball_pos.0 > block_pos.0 {
                Input::RIGHT
            } else {
                self.ball_momentum
            };
            // Recalculate inputs for unexpected bounce
            game.inputs.clear();
        }
        // Game is awaiting input and we have needed positional info
        if game.inputs.len() == 0 && ball_pos.is_some() && paddle_pos.is_some() {
            let ball_pos = ball_pos.unwrap();
            let paddle_pos = paddle_pos.unwrap();
            // Update ball momentum based on previous position
            self.ball_momentum = if ball_pos.0 < self.prev_ball_pos.0 {
                Input::LEFT
            } else if ball_pos.0 > self.prev_ball_pos.0 {
                Input::RIGHT
            } else {
                self.ball_momentum
            };
            // Check for wall bounces in the next frame and pre-emptively reverse ball momentum
            self.ball_momentum = match self.ball_momentum {
                Input::LEFT => match screen.get(&(ball_pos.0 - 1, ball_pos.1)) {
                    Some(Tile::WALL) => Input::RIGHT,
                    _ => self.ball_momentum,
                },
                Input::RIGHT => match screen.get(&(ball_pos.0 + 1, ball_pos.1)) {
                    Some(Tile::WALL) => Input::LEFT,
                    _ => self.ball_momentum,
                },
                _ => self.ball_momentum,
            };
            // Finally figure out the input based on the ball's position and momentum
            let input = if ball_pos.0 > paddle_pos.0 {
                // There is enough of a height difference to let the ball come to us
                if self.ball_momentum == Input::LEFT && paddle_pos.1 - ball_pos.1 > 1 {
                    Input::NONE
                } else {
                    Input::RIGHT
                }
            } else if ball_pos.0 < paddle_pos.0 {
                // There is enough of a height difference to let the ball come to us
                if self.ball_momentum == Input::RIGHT && paddle_pos.1 - ball_pos.1 > 1 {
                    Input::NONE
                } else {
                    Input::LEFT
                }
            } else {
                self.ball_momentum
            };
            self.prev_ball_pos = *ball_pos;
            self.prev_screen = screen.clone();
            game.push(&vec![input.to_i64()]);
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");
    println!("13-1:");
    let mut game = intcode::Cpu::new(rom.to_vec());
    let mut screen = Screen::new();
    loop {
        let (x, y, tile) = (game.run(), game.run(), game.run());
        match (x, y, tile) {
            (Some(x), Some(y), Some(tile)) => screen.insert((x, y), Tile::from(tile)),
            _ => break,
        };
    }
    println!("{}", screen.values().filter(|t| **t == Tile::BLOCK).count());
    println!("13-2:");
    let mut free_rom = rom.to_vec();
    free_rom[0] = 2;
    let mut game = intcode::Cpu::new(free_rom);
    game.push(&vec![Input::RIGHT.to_i64()]);
    let mut screen = Screen::new();
    let mut score = 0;
    let mut aimbot = BallPredictionEngine::new();
    loop {
        let (x, y, tile) = (game.run(), game.run(), game.run());
        match (x, y, tile) {
            (Some(-1), Some(0), Some(new_score)) => {
                score = new_score;
            }
            (Some(x), Some(y), Some(tile)) => {
                screen.insert((x, y), Tile::from(tile));
            }
            _ => break,
        };
        aimbot.move_paddle(&mut game, &screen);
    }
    println!("{}", score);
}
