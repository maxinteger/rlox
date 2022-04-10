#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
    EOP,
}

pub type Code = u8;

pub type Value = f64;

pub struct Chunk {
    pub code: Vec<Code>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn push_chunk(&mut self, code: Code, line: usize) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn push_op_code(&mut self, code: OpCode, line: usize) {
        self.push_chunk(code as Code, line)
    }

    pub fn get_op_code(&self, offset: usize) -> OpCode {
        if offset < self.code.len() {
            let chunk = self.code[offset];
            let op_code: OpCode = unsafe { ::std::mem::transmute(chunk) };
            op_code
        } else {
            OpCode::EOP
        }
    }

    pub fn push_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, idx: usize) -> Option<&Value> {
        self.constants.get(idx)
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        loop {
            if offset >= self.code.len() {
                break;
            }
            offset = self.disassemble_instruction(offset)
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        if self.code.is_empty() {
            return offset
        }

        print!("{:0>4} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{: >4} ", self.lines[offset]);
        }

        match self.get_op_code(offset) {
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::Add => self.simple_instruction("OP_ADD", offset),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => self.simple_instruction("OP_Divide", offset),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
            OpCode::EOP => self.simple_instruction("OP_END_OF_PROGRAM", offset),
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
