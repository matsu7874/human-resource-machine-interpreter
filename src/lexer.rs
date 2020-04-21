#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation<T> {
    pub value: T,
    pub location: Location,
}
impl<T> Annotation<T> {
    fn new(value: T, location: Location) -> Self {
        Self { value, location }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    InBox,
    OutBox,
    CopyFrom(usize),
    CopyTo(usize),
    Add(usize),
    Sub(usize),
    BumpPlus(usize),
    BumpMinus(usize),
    Jump(String),
    JumpIfZero(String),
    JumpIfNeg(String),
    JumpTarget(String),
}
pub type Token = Annotation<TokenKind>;
impl Token {
    pub fn inbox(location: Location) -> Self {
        Self::new(TokenKind::InBox, location)
    }
    pub fn outbox(location: Location) -> Self {
        Self::new(TokenKind::OutBox, location)
    }
    pub fn copy_from(location: Location, index: usize) -> Self {
        Self::new(TokenKind::CopyFrom(index), location)
    }
    pub fn copy_to(location: Location, index: usize) -> Self {
        Self::new(TokenKind::CopyTo(index), location)
    }
    pub fn add(location: Location, index: usize) -> Self {
        Self::new(TokenKind::Add(index), location)
    }
    pub fn sub(location: Location, index: usize) -> Self {
        Self::new(TokenKind::Sub(index), location)
    }
    pub fn bump_plus(location: Location, index: usize) -> Self {
        Self::new(TokenKind::BumpPlus(index), location)
    }
    pub fn bump_minus(location: Location, index: usize) -> Self {
        Self::new(TokenKind::BumpMinus(index), location)
    }
    pub fn jump(location: Location, label: String) -> Self {
        Self::new(TokenKind::Jump(label), location)
    }
    pub fn jump_if_zero(location: Location, label: String) -> Self {
        Self::new(TokenKind::JumpIfZero(label), location)
    }
    pub fn jump_if_neg(location: Location, label: String) -> Self {
        Self::new(TokenKind::JumpIfNeg(label), location)
    }
    pub fn jump_target(location: Location, label: String) -> Self {
        Self::new(TokenKind::JumpTarget(label), location)
    }
}

pub type Program = Vec<Token>;
macro_rules! require_arg {
    ($input: ident, $arg_type: ty, $index: expr) => {
        if $index < $input.len() {
            if let Ok(v) = $input[$index].value.parse::<$arg_type>() {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    };
}

pub struct Lexer;

impl Lexer {
    pub fn lex(input: &str) -> Program {
        let mut line = 1;
        let mut col = 1;
        let mut tokens = Vec::new();
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut end = 0;
        for c in input.chars() {
            let loc = Location { line, col };
            if c.is_ascii_whitespace() {
                if start < end {
                    chunks.push(Annotation::new(&input[start..end], loc));
                }
                col += end - start + 1;
                start = end + 1;
                if c == '\n' {
                    line += 1;
                    col = 1;
                }
            }
            end += 1;
        }
        // lexer allows program which terminated with '\n' or not '\n'.
        if start < end {
            chunks.push(Annotation::new(&input[start..end], Location { line, col }));
        }

        let mut i = 0;
        while i < chunks.len() {
            if let Some(command) = match chunks[i].value {
                "inbox" => {
                    let token = Token::inbox(chunks[i].location);
                    i += 1;
                    Some(token)
                }
                "outbox" => {
                    let token = Token::outbox(chunks[i].location);
                    i += 1;
                    Some(token)
                }
                "copyfrom" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::copy_from(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "copyto" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::copy_to(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "add" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::add(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "sub" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::sub(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "bump_plus" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::bump_plus(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "bump_minus" => {
                    if let Some(arg) = require_arg!(chunks, usize, i + 1) {
                        let token = Token::bump_minus(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "jump" => {
                    if let Some(arg) = require_arg!(chunks, String, i + 1) {
                        let token = Token::jump(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "jump_if_zero" => {
                    if let Some(arg) = require_arg!(chunks, String, i + 1) {
                        let token = Token::jump_if_zero(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "jump_if_neg" => {
                    if let Some(arg) = require_arg!(chunks, String, i + 1) {
                        let token = Token::jump_if_neg(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                "jump_target" => {
                    if let Some(arg) = require_arg!(chunks, String, i + 1) {
                        let token = Token::jump_target(chunks[i + 1].location, arg);
                        i += 2;
                        Some(token)
                    } else {
                        None
                    }
                }
                _ => {
                    i += 1;
                    None
                }
            } {
                tokens.push(command);
            } else {
                i += 1;
            }
            line += 1;
        }
        tokens
    }
}
