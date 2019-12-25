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
    STOP
}

const OPCODE_LENGTH: usize = 1;

#[derive(Debug)]
enum OpParamMode {
    POINTER,
    VALUE
}

#[derive(Debug)]
struct OpParam {
    mode: OpParamMode,
    offset: usize
}

impl OpParam {
    fn new(mode: OpParamMode, offset: usize) -> OpParam {
        OpParam{ mode, offset }
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
        Op{ code, params }
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

#[derive(Default)]
struct Cpu {
    memory: Vec<i64>,
    ax: i64,
    bx: i64,
    cx: i64,
    sp: usize,
}

impl Cpu {
    fn new(memory: Vec<i64>) -> Cpu {
        Cpu { memory, ..Default::default() }
    }

    fn clear_registers(&mut self) {
        self.ax = 0;
        self.bx = 0;
        self.cx = 0;
        self.sp = 0;
    }

    fn run(&mut self, input: i64) -> Vec<i64> {
        self.clear_registers();
        let mut outputs: Vec<i64> = vec![];

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

                    self.memory[self.ax as usize] = input;
                    self.sp += op.len();
                }
                OpCode::OUT => {
                    self.ax = op.params[0].read(&self.memory, self.sp);

                    outputs.push(self.ax);
                    self.sp += op.len();
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
                OpCode::STOP => break
            }
        }
        outputs
    }
}

fn main() {
    let rom = parse_rom("input");
    println!("5-1:");
    println!("{:?}", Cpu::new(rom.to_vec()).run(1));
    println!("5-2:");
    println!("{:?}", Cpu::new(rom.to_vec()).run(5));
}
