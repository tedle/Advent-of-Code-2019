extern crate intcode;

use std::collections::{HashMap, VecDeque};

struct Packet {
    data: Vec<i64>,
}

impl Packet {
    fn new() -> Packet {
        Packet { data: vec![] }
    }

    fn push(&mut self, value: i64) -> Option<(i64, i64, i64)> {
        self.data.push(value);
        match self.data.len() {
            3 => Some((self.data[0], self.data[1], self.data[2])),
            1..=2 => None,
            _ => panic!("Packet overflowed"),
        }
    }
}

fn run_network(nic: &Vec<i64>, early_return: bool) -> i64 {
    let mut network = vec![];
    let mut packet_queue: HashMap<usize, VecDeque<(i64, i64)>> = HashMap::new();
    let mut nat_packet: Option<(i64, i64)> = None;
    let mut last_sent_nat_packet: Option<(i64, i64)> = None;
    for i in 0..50 {
        network.push((intcode::Cpu::new(nic.to_vec()), Packet::new()));
        packet_queue.insert(i, VecDeque::new());
        // Initialize network address
        network[i].0.push(&vec![i as i64]);
    }
    let mut idle_count = 0;
    loop {
        if idle_count == network.len() && nat_packet.is_some() {
            if let Some((_, y)) = last_sent_nat_packet {
                if y == nat_packet.unwrap().1 {
                    return y;
                }
            }
            packet_queue
                .get_mut(&0)
                .unwrap()
                .push_back(nat_packet.unwrap());
            last_sent_nat_packet = nat_packet;
            nat_packet = None;
        }

        idle_count = 0;
        for (i, (cpu, next_packet)) in network.iter_mut().enumerate() {
            let queue = &mut packet_queue.get_mut(&i).unwrap();
            for (x, y) in queue.iter() {
                cpu.push(&vec![*x, *y]);
            }
            queue.clear();
            if cpu.inputs.len() == 0 {
                cpu.inputs.push_back(-1);
            }
            if cpu.inputs.len() == 1 && cpu.inputs[0] == -1 {
                idle_count += 1;
            }

            match cpu.poll() {
                intcode::Poll::Result(i) => {
                    if let Some((a, x, y)) = next_packet.push(i) {
                        if a == 255 {
                            if early_return {
                                return y;
                            }
                            nat_packet = Some((x, y));
                        } else {
                            packet_queue
                                .get_mut(&(a as usize))
                                .unwrap()
                                .push_back((x, y));
                        }
                        *next_packet = Packet::new();
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let rom = intcode::parse_rom("input");

    println!("23-1:\n{}", run_network(&rom, true));
    println!("23-2:\n{}", run_network(&rom, false));
}
