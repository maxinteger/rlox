#[derive(Copy, Clone)]

#[repr(u8)]
enum OpCode {
    NoOp,
    OpConstant,
    OpReturn
}

type Code = u8;

type Value = f64;

struct Chunk {
    code: Vec<Code>,
    constants: Vec<Value>,
    lines: Vec<usize>
}

impl Chunk {
    fn new () ->Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new()
        }
    }

    fn push_chunk(&mut self, code: Code, line: usize) {
        self.code.push(code);
        self.lines.push(line);
    }

    fn push_op_code(&mut self, code: OpCode, line: usize) {
        self.push_chunk(code as Code, line)
    }

    fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1).try_into().unwrap()
    }

    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        loop {
            if offset >= self.code.len() { break }
            offset = self.disassemble_instruction(offset)
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:0>4} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{: >4} ", self.lines[offset]);
        }


        let chunk = self.code[offset];
        let instruction: OpCode = unsafe { ::std::mem::transmute(chunk) };

        match instruction {
            OpCode::OpReturn => {
                self.simple_instruction("OP_RETURN", offset)
            }
            OpCode::OpConstant => {
                self.constant_instruction("OP_CONSTANT", offset)
            }
            _ => {
                println!("Unknown opcode {}\n", chunk);
                offset + 1
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let const_idx = self.code[offset + 1] as usize;
        let constant = self.constants[const_idx];
        println!("{: <16} {: >4} '{}'", name, offset, constant);
        offset + 2
    }
}

fn main() {
    let mut chunks = Chunk::new();
    let constant = chunks.add_constant(1.2);
    chunks.push_op_code(OpCode::OpConstant, 123);
    chunks.push_chunk(constant, 123);
    chunks.push_op_code(OpCode::OpReturn, 123);
    chunks.disassemble("Test");
}
