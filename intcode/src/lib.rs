use std::collections::VecDeque;
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

pub fn parse_rom(filename: &str) -> Vec<i64> {
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

pub enum Poll {
    Result(i64),
    None,
    Stop,
}

impl Poll {
    pub fn to_option(&self) -> Option<i64> {
        match self {
            Poll::Result(i) => Some(*i),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Cpu {
    pub inputs: VecDeque<i64>,
    memory: Memory,
    ax: i64,
    bx: i64,
    cx: i64,
    sp: usize,
    bp: usize,
}

impl Cpu {
    pub fn new(memory: Vec<i64>) -> Cpu {
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

    pub fn push(&mut self, input: &Vec<i64>) {
        self.inputs.extend(input.iter().cloned());
    }

    fn read_param(&self, param: &OpParam) -> i64 {
        param.read(&self.memory, self.bp, self.sp)
    }

    fn read_param_as_ptr(&self, param: &OpParam) -> i64 {
        param.read_as_ptr(&self.memory, self.bp, self.sp)
    }

    pub fn poll(&mut self) -> Poll {
        let op = Op::new(&self.memory, self.sp);
        self.run_op(&op)
    }

    pub fn run(&mut self) -> Option<i64> {
        self.run_with(&vec![])
    }

    pub fn run_with(&mut self, input: &Vec<i64>) -> Option<i64> {
        self.push(input);

        loop {
            let op = Op::new(&self.memory, self.sp);
            match self.run_op(&op) {
                Poll::Result(output) => return Some(output),
                Poll::Stop => break,
                Poll::None => (),
            }
        }
        None
    }

    fn run_op(&mut self, op: &Op) -> Poll {
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
                return Poll::Result(self.ax);
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
            OpCode::Stop => return Poll::Stop,
        }
        Poll::None
    }
}
