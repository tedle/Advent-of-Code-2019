use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, VecDeque};
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
    cpu: Cpu,
    origin: Position,
    unsolved: BTreeMap<i64, VecDeque<Position>>,
    map: Map,
}

impl LocatorRobot {
    fn new(cpu: Cpu) -> LocatorRobot {
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

    fn move_from_origin_to(&self, pos: Position, robot: &mut Cpu) {
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
    let rom = parse_rom("input");
    println!("15-1:");
    let mut robot = LocatorRobot::new(Cpu::new(rom.to_vec()));
    robot.solve();
    let (pos, goal) = robot
        .map
        .iter()
        .filter(|(_, t)| t.kind == TileKind::GOAL)
        .next()
        .unwrap();
    println!("{}", goal.cost.unwrap());
    println!("15-2:");
    let mut cpu = Cpu::new(rom.to_vec());
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
