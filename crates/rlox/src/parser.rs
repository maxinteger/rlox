use std::collections::HashMap;
use std::str::FromStr;
use crate::{Chunk, Code, OpCode, Scanner, scanner, Token, TokenType, Value};

pub struct Parser<'a> {
    scanner: &'a mut Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    chunk: &'a mut Chunk,
    rules: HashMap<TokenType, ParseRule>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut scanner::Scanner<'a>, chunk: &'a mut Chunk) -> Self {
        Parser {
            previous: Token::default(),
            current: Token::default(),
            scanner,
            had_error: false,
            panic_mode: false,
            chunk,
            rules: Parser::get_rules(),
        }
    }

    pub fn parse(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end();
        !self.had_error
    }
    fn get_rules() -> HashMap<TokenType, ParseRule> {
        let mut map = HashMap::new();
        use TokenType::*;

        map.insert(
            LeftParen,
            ParseRule::new(ParseFn::Groping, ParseFn::None, Precedence::None),
        );
        map.insert(
            RightParen,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            LeftBrace,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            RightBrace,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Comma,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Dot,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Minus,
            ParseRule::new(ParseFn::Unary, ParseFn::Binary, Precedence::Term),
        );
        map.insert(
            Plus,
            ParseRule::new(ParseFn::None, ParseFn::Binary, Precedence::Term),
        );
        map.insert(
            Semicolon,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Slash,
            ParseRule::new(ParseFn::None, ParseFn::Binary, Precedence::Factor),
        );
        map.insert(
            Star,
            ParseRule::new(ParseFn::None, ParseFn::Binary, Precedence::Factor),
        );
        map.insert(
            Bang,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            BangEqual,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Equal,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            EqualEqual,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Greater,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            GreaterEqual,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Less,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            LessEqual,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Identifier,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            String,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Number,
            ParseRule::new(ParseFn::Number, ParseFn::None, Precedence::None),
        );
        map.insert(
            And,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Class,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Else,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            False,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            For,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Fun,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            If,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Nil,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Or,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Print,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Return,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Super,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            This,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            True,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Var,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            While,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Error,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map.insert(
            Eof,
            ParseRule::new(ParseFn::None, ParseFn::None, Precedence::None),
        );
        map
    }

    fn emit_byte(&mut self, byte: Code) {
        self.chunk.push_chunk(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: Code, byte2: Code) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return as Code);

        if cfg!(feature = "debug_print_code") && !self.had_error {
            self.chunk.disassemble("code");
        }
    }

    fn advance(&mut self) {
        self.previous = self.current;

        loop {
            self.current = self.scanner.scan_token();
            if self.current.kind != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.src)
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(&Precedence::Assignment)
    }

    fn consume(&mut self, kind: TokenType, message: &str) {
        if self.current.kind == kind {
            self.advance()
        } else {
            self.error_at_current(message)
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.previous, message)
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.kind == TokenType::Eof {
            eprint!(" at end");
        } else if token.kind == TokenType::Error {
            todo!()
        } else {
            eprint!(" at '{}'", token.src)
        }
        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn number(&mut self) {
        let value: Value = Value::from_str(self.previous.src).unwrap();
        self.emit_constant(value)
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as Code, constant)
    }
    fn make_constant(&mut self, value: Value) -> Code {
        let constant_idx = self.chunk.push_constant(value);
        if constant_idx > u8::MAX.into() {
            self.error_at_current("Too many constants in one chunk")
        }
        constant_idx.try_into().unwrap()
    }

    fn parse_precedence(&mut self, precedence: &Precedence) {
        self.advance();
        let token = self.previous.kind;
        let prefix_rule = self.get_rule(&token);

        self.execute_parse_fn(&prefix_rule.prefix, true);

        while precedence <= &self.get_rule(&self.current.kind).precedence {
            self.advance();
            let infix_rule = &self.get_rule(&self.previous.kind).infix;
            self.execute_parse_fn(infix_rule, false)
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    fn unary(&mut self) {
        let op_kind = self.previous.kind;

        self.expression();

        match op_kind {
            TokenType::Minus => self.emit_byte(OpCode::Negate as Code),
            _ => return,
        }
    }

    fn binary(&mut self) {
        let op_kind = self.previous.kind;
        let rule = self.get_rule(&op_kind);
        self.parse_precedence(&rule.get_next_precedence());

        match op_kind {
            TokenType::Plus => self.emit_byte(OpCode::Add as Code),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as Code),
            TokenType::Star => self.emit_byte(OpCode::Multiply as Code),
            TokenType::Slash => self.emit_byte(OpCode::Divide as Code),
            _ => {},
        }
    }

    fn get_rule(&self, token_type: &TokenType) -> ParseRule {
        *self.rules.get(token_type).expect("Missing parser rule")
    }

    fn execute_parse_fn(&mut self, fun: &ParseFn, strict: bool) {
        match fun {
            ParseFn::None => {
                if strict {
                    self.error_at_current("Expect expression.")
                }
            }
            ParseFn::Groping => self.grouping(),
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Number => self.number(),
        }
    }
}



#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum ParseFn {
    None,
    Groping,
    Unary,
    Binary,
    Number,
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

impl ParseRule {
    fn new(prefix: ParseFn, infix: ParseFn, precedence: Precedence) -> Self {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }

    fn get_next_precedence(&self) -> Precedence {
        let prec = &self.precedence;
        let next = if prec == &Precedence::Primary {
            (*prec) as u8
        } else {
            ((*prec) as u8) + 1
        };
        let precedence: Precedence = unsafe { ::std::mem::transmute(next) };
        precedence
    }
}
