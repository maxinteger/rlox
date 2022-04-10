use crate::chunk::{Chunk, OpCode, Value};
use crate::{InterpretResult, VmStack};

pub struct VirtualMachine {
    pub chunks: Chunk,
    stack: VmStack<Value>,
    ip: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            chunks: Chunk::new(),
            stack: VmStack::new(256),
            ip: 0,
        }
    }

    fn interpret(&mut self) -> InterpretResult {
        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            if cfg!(feature = "debug_trace_execution") {
                self.stack.trace();
                self.chunks.disassemble_instruction(self.ip);
            }
            match self.get_next_op_code() {
                OpCode::Constant => {
                    let idx = self.get_next_byte();
                    if let Some(value) = self.chunks.get_constant(idx as usize) {
                        self.stack.push(*value)
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                    continue;
                }
                OpCode::Add => {
                    let value_a = self.stack.pop();
                    let value_b = self.stack.pop();
                    if value_a != None && value_b != None {
                        self.stack.push(value_a.unwrap() + value_b.unwrap());
                        continue;
                    }
                    return InterpretResult::RuntimeError;
                }
                OpCode::Subtract => {
                    let value_a = self.stack.pop();
                    let value_b = self.stack.pop();
                    if value_a != None && value_b != None {
                        self.stack.push(value_a.unwrap() - value_b.unwrap());
                        continue;
                    }
                    return InterpretResult::RuntimeError;
                }
                OpCode::Multiply => {
                    let value_a = self.stack.pop();
                    let value_b = self.stack.pop();
                    if value_a != None && value_b != None {
                        self.stack.push(value_a.unwrap() * value_b.unwrap());
                        continue;
                    }
                    return InterpretResult::RuntimeError;
                }
                OpCode::Divide => {
                    let value_a = self.stack.pop();
                    let value_b = self.stack.pop();
                    if value_a != None && value_b != None {
                        self.stack.push(value_a.unwrap() / value_b.unwrap());
                        continue;
                    }
                    return InterpretResult::RuntimeError;
                }
                OpCode::Negate => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(-value);
                        continue;
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Return => {
                    println!("{:?}", self.stack.pop().unwrap());
                    break;
                }
                OpCode::EOP => {
                    break;
                }
            }
        }
        InterpretResult::Ok
    }

    fn get_next_op_code(&mut self) -> OpCode {
        let code = self.chunks.get_op_code(self.ip);
        self.ip += 1;
        code
    }

    fn get_next_byte(&mut self) -> u8 {
        let byte = self.chunks.code[self.ip];
        self.ip += 1;
        byte
    }
}
