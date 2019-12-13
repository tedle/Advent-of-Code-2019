use std::cmp::{max, min};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
enum OpCode {
    Add,
    Mul,
    In,
    Out,
    Jnz,
    Jz,
    Lt,
    Eq,
    AddBp,
    Stop,
}

#[derive(Debug)]
enum OpParamMode {
    POINTER,
    VALUE,
    RELATIVE,
}

#[derive(Debug)]
struct OpParam {
    mode: OpParamMode,
    offset: usize,
}

impl OpParam {
    fn new(mode: OpParamMode, offset: usize) -> OpParam {
        OpParam { mode, offset }
    }

    fn read(&self, memory: &Memory, base: usize, address: usize) -> i64 {
        match &self.mode {
            OpParamMode::POINTER => {
                memory[memory[address + self.offset + Op::OPCODE_LENGTH] as usize]
            }
            OpParamMode::RELATIVE => {
                memory[(base as i64 + memory[address + self.offset + Op::OPCODE_LENGTH]) as usize]
            }
            OpParamMode::VALUE => memory[address + self.offset + Op::OPCODE_LENGTH],
        }
    }

    fn read_as_ptr(&self, memory: &Memory, base: usize, address: usize) -> i64 {
        match &self.mode {
            OpParamMode::POINTER => memory[address + self.offset + Op::OPCODE_LENGTH],
            OpParamMode::RELATIVE => {
                memory[address + self.offset + Op::OPCODE_LENGTH] + base as i64
            }
            OpParamMode::VALUE => panic!("Value parameters cannot be written to"),
        }
    }
}

#[derive(Debug)]
struct Op {
    code: OpCode,
    params: Vec<OpParam>,
}

impl Op {
    fn new(memory: &Memory, address: usize) -> Op {
        let code = memory[address] % 100; // Last 2 digits
        let parameter_modes = (memory[address] / 100)
            .to_string()
            .chars()
            .map(|n| n.to_digit(10).unwrap() as usize)
            .rev()
            .collect::<Vec<_>>(); // Preceeding digits
        fn parse_parameter(modes: &Vec<usize>, param_count: usize) -> Vec<OpParam> {
            let mut params: Vec<OpParam> = vec![];
            for i in 0..param_count {
                let mode = match modes.get(i).unwrap_or(&0) {
                    0 => OpParamMode::POINTER,
                    1 => OpParamMode::VALUE,
                    2 => OpParamMode::RELATIVE,
                    _ => panic!("Unknown parameter mode"),
                };
                params.push(OpParam::new(mode, i));
            }
            params
        };
        let (code, parameter_count) = match code {
            1 => (OpCode::Add, 3),
            2 => (OpCode::Mul, 3),
            3 => (OpCode::In, 1),
            4 => (OpCode::Out, 1),
            5 => (OpCode::Jnz, 2),
            6 => (OpCode::Jz, 2),
            7 => (OpCode::Lt, 3),
            8 => (OpCode::Eq, 3),
            9 => (OpCode::AddBp, 1),
            99 => (OpCode::Stop, 0),
            _ => panic!("Unknown op: {}", code),
        };
        Op {
            code,
            params: parse_parameter(&parameter_modes, parameter_count),
        }
    }

    fn len(&self) -> usize {
        Op::OPCODE_LENGTH + self.params.len()
    }

    const OPCODE_LENGTH: usize = 1;
}

fn parse_rom(filename: &str) -> Vec<i64> {
    let input = fs::read_to_string(filename).unwrap();
    input
        .split(",")
        .map(|op| op.trim().parse().unwrap())
        .collect()
}

#[derive(Clone)]
struct Memory {
    data: Vec<i64>,
    max_len: usize,
}

impl Memory {
    fn new(max_len: usize) -> Memory {
        Memory {
            data: vec![],
            max_len,
        }
    }
    fn from(data: Vec<i64>, max_len: usize) -> Memory {
        let mut memory = Memory::new(max_len);
        memory.data = data;
        memory
    }
    const UNINITIALIZED: i64 = 0;
}

impl Index<usize> for Memory {
    type Output = i64;

    fn index(&self, i: usize) -> &Self::Output {
        if i > self.max_len - 1 {
            panic!("Exceeded maximum RAM")
        }
        if i > self.data.len() - 1 {
            return &Memory::UNINITIALIZED;
        }
        &self.data[i]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        if i > self.max_len - 1 {
            panic!("Exceeded maximum RAM")
        }
        if i > self.data.len() - 1 {
            self.data.resize_with(i + 1, || Memory::UNINITIALIZED);
        }
        &mut self.data[i]
    }
}

#[derive(Clone)]
struct Cpu {
    memory: Memory,
    inputs: VecDeque<i64>,
    ax: i64,
    bx: i64,
    cx: i64,
    sp: usize,
    bp: usize,
}

impl Cpu {
    fn new(memory: Vec<i64>) -> Cpu {
        Cpu {
            memory: Memory::from(memory, 1024 * 1024),
            inputs: VecDeque::new(),
            ax: 0,
            bx: 0,
            cx: 0,
            bp: 0,
            sp: 0,
        }
    }

    fn push(&mut self, input: &Vec<i64>) {
        self.inputs.extend(input.iter().cloned());
    }

    fn read_param(&self, param: &OpParam) -> i64 {
        param.read(&self.memory, self.bp, self.sp)
    }

    fn read_param_as_ptr(&self, param: &OpParam) -> i64 {
        param.read_as_ptr(&self.memory, self.bp, self.sp)
    }

    fn run(&mut self) -> Option<i64> {
        self.run_with(&vec![])
    }

    fn run_with(&mut self, input: &Vec<i64>) -> Option<i64> {
        self.push(input);

        loop {
            let op = Op::new(&self.memory, self.sp);
            match op.code {
                OpCode::Add => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);
                    self.cx = self.read_param_as_ptr(&op.params[2]);

                    self.memory[self.cx as usize] = self.ax + self.bx;
                    self.sp += op.len();
                }
                OpCode::Mul => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);
                    self.cx = self.read_param_as_ptr(&op.params[2]);

                    self.memory[self.cx as usize] = self.ax * self.bx;
                    self.sp += op.len();
                }
                OpCode::In => {
                    self.ax = self.read_param_as_ptr(&op.params[0]);

                    self.memory[self.ax as usize] =
                        self.inputs.pop_front().expect("Missing input parameter");
                    self.sp += op.len();
                }
                OpCode::Out => {
                    self.ax = self.read_param(&op.params[0]);

                    self.sp += op.len();
                    return Some(self.ax);
                }
                OpCode::Jnz => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);

                    self.sp = match self.ax {
                        0 => self.sp + op.len(),
                        _ => self.bx as usize,
                    };
                }
                OpCode::Jz => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);

                    self.sp = match self.ax {
                        0 => self.bx as usize,
                        _ => self.sp + op.len(),
                    };
                }
                OpCode::Lt => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);
                    self.cx = self.read_param_as_ptr(&op.params[2]);

                    self.memory[self.cx as usize] = if self.ax < self.bx { 1 } else { 0 };
                    self.sp += op.len();
                }
                OpCode::Eq => {
                    self.ax = self.read_param(&op.params[0]);
                    self.bx = self.read_param(&op.params[1]);
                    self.cx = self.read_param_as_ptr(&op.params[2]);

                    self.memory[self.cx as usize] = if self.ax == self.bx { 1 } else { 0 };
                    self.sp += op.len();
                }
                OpCode::AddBp => {
                    self.ax = self.read_param(&op.params[0]);

                    self.bp = (self.bp as i64 + self.ax) as usize;
                    self.sp += op.len();
                }
                OpCode::Stop => break,
            }
        }
        None
    }
}

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

    fn move_paddle(&mut self, game: &mut Cpu, screen: &Screen) {
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
    let rom = parse_rom("input");
    println!("13-1:");
    let mut game = Cpu::new(rom.to_vec());
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
    let mut game = Cpu::new(free_rom);
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
