use std::fs;

#[derive(Debug)]
enum OpCode {
    ADD,
    MUL,
    IN,
    OUT,
    JNZ,
    JZ,
    LT,
    EQ,
    STOP,
}

const OPCODE_LENGTH: usize = 1;

#[derive(Debug)]
enum OpParamMode {
    POINTER,
    VALUE,
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

    fn read(&self, memory: &Vec<i64>, address: usize) -> i64 {
        match &self.mode {
            OpParamMode::POINTER => memory[memory[address + self.offset + OPCODE_LENGTH] as usize],
            OpParamMode::VALUE => memory[address + self.offset + OPCODE_LENGTH],
        }
    }
}

#[derive(Debug)]
struct Op {
    code: OpCode,
    params: Vec<OpParam>,
}

impl Op {
    fn new(memory: &Vec<i64>, address: usize) -> Op {
        let code = memory[address] % 100; // Last 2 digits
        let parameter_modes = (memory[address] / 100)
            .to_string()
            .chars()
            .map(|n| n.to_digit(10).unwrap() as usize)
            .rev()
            .collect::<Vec<_>>(); // Preceeding digits
        let parse_parameter = |modes: &Vec<usize>, index: usize| -> OpParam {
            let mode = match modes.get(index).unwrap_or(&0) {
                0 => OpParamMode::POINTER,
                1 => OpParamMode::VALUE,
                _ => panic!("Unknown parameter mode"),
            };
            OpParam::new(mode, index)
        };
        let (code, params) = match code {
            1 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);
                let cx = OpParam::new(OpParamMode::VALUE, 2);

                (OpCode::ADD, vec![ax, bx, cx])
            }
            2 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);
                let cx = OpParam::new(OpParamMode::VALUE, 2);

                (OpCode::MUL, vec![ax, bx, cx])
            }
            3 => {
                let ax = OpParam::new(OpParamMode::VALUE, 0);

                (OpCode::IN, vec![ax])
            }
            4 => {
                let ax = parse_parameter(&parameter_modes, 0);

                (OpCode::OUT, vec![ax])
            }
            5 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);

                (OpCode::JNZ, vec![ax, bx])
            }
            6 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);

                (OpCode::JZ, vec![ax, bx])
            }
            7 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);
                let cx = OpParam::new(OpParamMode::VALUE, 2);

                (OpCode::LT, vec![ax, bx, cx])
            }
            8 => {
                let ax = parse_parameter(&parameter_modes, 0);
                let bx = parse_parameter(&parameter_modes, 1);
                let cx = OpParam::new(OpParamMode::VALUE, 2);

                (OpCode::EQ, vec![ax, bx, cx])
            }
            99 => (OpCode::STOP, vec![]),
            _ => {
                panic!("Unknown op: {}", code);
            }
        };
        Op { code, params }
    }

    fn len(&self) -> usize {
        OPCODE_LENGTH + self.params.len()
    }
}

fn parse_rom(filename: &str) -> Vec<i64> {
    let input = fs::read_to_string(filename).unwrap();
    input
        .split(",")
        .map(|op| op.trim().parse().unwrap())
        .collect()
}

#[derive(Clone, Default)]
struct Cpu {
    memory: Vec<i64>,
    inputs: std::collections::VecDeque<i64>,
    ax: i64,
    bx: i64,
    cx: i64,
    sp: usize,
}

impl Cpu {
    fn new(memory: Vec<i64>) -> Cpu {
        Cpu {
            memory,
            ..Default::default()
        }
    }

    fn push(&mut self, input: &Vec<i64>) {
        self.inputs.extend(input.iter().cloned());
    }

    fn run(&mut self, input: &Vec<i64>) -> Option<i64> {
        self.inputs.extend(input.iter().cloned());

        loop {
            let op = Op::new(&self.memory, self.sp);
            match op.code {
                OpCode::ADD => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);
                    self.cx = op.params[2].read(&self.memory, self.sp);

                    self.memory[self.cx as usize] = self.ax + self.bx;
                    self.sp += op.len();
                }
                OpCode::MUL => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);
                    self.cx = op.params[2].read(&self.memory, self.sp);

                    self.memory[self.cx as usize] = self.ax * self.bx;
                    self.sp += op.len();
                }
                OpCode::IN => {
                    self.ax = op.params[0].read(&self.memory, self.sp);

                    self.memory[self.ax as usize] =
                        self.inputs.pop_front().expect("Missing input parameter");
                    self.sp += op.len();
                }
                OpCode::OUT => {
                    self.ax = op.params[0].read(&self.memory, self.sp);

                    self.sp += op.len();
                    return Some(self.ax);
                }
                OpCode::JNZ => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);

                    self.sp = match self.ax {
                        0 => self.sp + op.len(),
                        _ => self.bx as usize,
                    };
                }
                OpCode::JZ => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);

                    self.sp = match self.ax {
                        0 => self.bx as usize,
                        _ => self.sp + op.len(),
                    };
                }
                OpCode::LT => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);
                    self.cx = op.params[2].read(&self.memory, self.sp);

                    self.memory[self.cx as usize] = if self.ax < self.bx { 1 } else { 0 };
                    self.sp += op.len();
                }
                OpCode::EQ => {
                    self.ax = op.params[0].read(&self.memory, self.sp);
                    self.bx = op.params[1].read(&self.memory, self.sp);
                    self.cx = op.params[2].read(&self.memory, self.sp);

                    self.memory[self.cx as usize] = if self.ax == self.bx { 1 } else { 0 };
                    self.sp += op.len();
                }
                OpCode::STOP => break,
            }
        }
        None
    }
}

/// Shamelessly stolen from: https://en.wikipedia.org/wiki/Heap%27s_algorithm
/// In retrospect figuring out a naive algorithm might've been more fun and less effort
fn permutations(input: Vec<i64>) -> Vec<Vec<i64>> {
    let mut output: Vec<Vec<i64>> = vec![];
    fn heaps(k: usize, array: &mut Vec<i64>, out: &mut Vec<Vec<i64>>) {
        if k == 1 {
            out.push(array.to_vec());
            return;
        }
        heaps(k - 1, array, out);
        for i in 0..k - 1 {
            if k % 2 == 0 {
                array.swap(i, k - 1);
            } else {
                array.swap(0, k - 1);
            }
            heaps(k - 1, array, out);
        }
    };
    heaps(input.len(), &mut input.to_vec(), &mut output);
    output
}

fn main() {
    let rom = parse_rom("input");
    println!("7-1:");
    let mut max_output = 0;
    for inputs in permutations((0..5).collect()) {
        let mut output = 0;
        for i in inputs {
            output = Cpu::new(rom.to_vec()).run(&vec![i, output]).unwrap();
        }
        max_output = std::cmp::max(max_output, output);
    }
    println!("{}", max_output);
    println!("7-2:");
    let mut max_output = 0;
    for inputs in permutations((5..10).collect()) {
        let mut amplifiers: Vec<Cpu> = vec![Cpu::new(rom.to_vec()); 5];
        let mut output = 0;
        for i in 0..5 {
            amplifiers[i].push(&vec![inputs[i]]);
        }
        'feedback_loop: loop {
            for i in 0..5 {
                match amplifiers[i].run(&vec![output]) {
                    Some(out) => output = out,
                    None => break 'feedback_loop,
                }
            }
        }
        max_output = std::cmp::max(max_output, output);
    }
    println!("{}", max_output);
}
