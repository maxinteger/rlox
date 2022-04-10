use crate::chunk::{Chunk, Code, OpCode, Value};
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::vm::VirtualMachine;
use crate::parser::Parser;
use std::io::BufRead;
use std::{env, fs, io};

mod chunk;
mod scanner;
mod parser;
mod token;
mod vm;

enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

struct VmStack<TValue: std::fmt::Debug> {
    max_size: usize,
    data: Vec<TValue>,
}

impl<TValue: std::fmt::Debug> VmStack<TValue> {
    fn new(max_size: usize) -> Self {
        VmStack {
            max_size,
            data: Vec::with_capacity(max_size),
        }
    }

    fn push(&mut self, value: TValue) {
        if self.data.len() >= self.max_size {
            panic!("Stack overflow")
        }
        self.data.push(value)
    }

    fn pop(&mut self) -> Option<TValue> {
        self.data.pop()
    }

    fn trace(&self) {
        for val in self.data.iter() {
            print!("[{:?}]", val);
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut vm = VirtualMachine::new();

    match args.len() {
        1 => {
            repl(&mut vm);
        }
        2 => {
            run_file(&mut vm, args[1].as_str());
        }
        _ => {
            println!("Usage: rlox [path]");

            std::process::exit(64);
        }
    }
}

fn repl(vm: &mut VirtualMachine) {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("> ");
        if let Ok(line) = lines.next().unwrap() {
            interpret(vm, line.as_str());
        } else {
            break;
        }
    }
}

fn run_file(vm: &mut VirtualMachine, path: &str) {
    if let Ok(source) = fs::read_to_string(path) {
        match interpret(vm, source.as_str()) {
            InterpretResult::Ok => {}
            InterpretResult::CompileError => {
                eprintln!("Compilation error");
                std::process::exit(65);
            }
            InterpretResult::RuntimeError => {
                eprintln!("Runtime error");
                std::process::exit(70);
            }
        }
    } else {
        eprintln!("Could not open file '{}'", path);
        std::process::exit(64);
    }
}

fn interpret(vm: &mut VirtualMachine, source: &str) -> InterpretResult {
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(&mut scanner, &mut vm.chunks);
    let parse_result = parser.parse();

    if !parse_result {
        return InterpretResult::CompileError;
    }

    vm.run()
}