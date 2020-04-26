use crate::lexer::{Annotation, Program, Token, TokenKind};
use log::{debug, trace};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpreterErrorKind {
    UnexistedJumpTarget,
    UndefinedInputBox,
    EmptyInBox,
    EmptyFloorValue,
    EmptyHandValue,
}
type InterpreterError = Annotation<InterpreterErrorKind>;

pub struct SimpleInterpreter {
    hand: Option<i16>,
    program_cursor: usize,
    cells: Vec<Option<i16>>,
    program: Program,
    inbox: Option<VecDeque<i16>>,
    jump_table: HashMap<usize, usize>,
    step_counter: usize,
}

impl SimpleInterpreter {
    pub fn new() -> Self {
        Self {
            hand: None,
            program_cursor: 0,
            cells: vec![],
            program: Vec::new(),
            inbox: None,
            jump_table: HashMap::new(),
            step_counter: 0,
        }
    }
    pub fn set_inbox(&mut self, inbox: String) {
        let mut stream = VecDeque::new();
        let mut n = None;
        let mut minus_flag = false;
        for c in inbox.bytes() {
            match c {
                b'0'..=b'9' => match n.as_mut() {
                    Some(v) => {
                        *v *= 10;
                        *v += u8::from_be(c) - u8::from_be(b'0');
                    }
                    None => {
                        n = Some(u8::from_be(c) - u8::from_be(b'0'));
                    }
                },
                _ => {
                    if let Some(v) = n {
                        if minus_flag {
                            stream.push_back(-1 * v as i16);
                        } else {
                            stream.push_back(v as i16);
                        }
                    }
                    n = None;
                    if c == b'-' {
                        minus_flag = true;
                    }
                }
            }
        }
        self.inbox = Some(stream);
    }
    pub fn show_inbox(&self) -> &Option<VecDeque<i16>> {
        &self.inbox
    }
    fn eval_inbox(&mut self, command: &Token) -> Result<(), InterpreterError> {
        self.hand = if let Some(ref mut input) = self.inbox {
            if input.len() > 0 {
                Some(input.pop_front().unwrap())
            } else {
                return Err(InterpreterError {
                    value: InterpreterErrorKind::EmptyInBox,
                    location: command.location,
                });
            }
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::UndefinedInputBox,
                location: command.location,
            });
        };
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_outbox(&mut self, command: &Token) -> Result<(), InterpreterError> {
        if let Some(value) = self.hand {
            println!("{}", value);
            self.hand = None;
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyHandValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_copy_from(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        if let Some(_value) = self.cells[index] {
            self.hand = self.cells[index];
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyFloorValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_copy_to(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        if let Some(value) = self.hand {
            self.cells[index] = Some(value);
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyHandValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }

    fn eval_add(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        if let Some(floor_value) = self.cells[index] {
            if let Some(ref mut hand_value) = self.hand {
                *hand_value += floor_value;
            } else {
                return Err(InterpreterError {
                    value: InterpreterErrorKind::EmptyHandValue,
                    location: command.location,
                });
            }
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyFloorValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_sub(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        if let Some(floor_value) = self.cells[index] {
            if let Some(ref mut hand_value) = self.hand {
                *hand_value -= floor_value;
            } else {
                return Err(InterpreterError {
                    value: InterpreterErrorKind::EmptyHandValue,
                    location: command.location,
                });
            }
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyFloorValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_bump(
        &mut self,
        command: &Token,
        index: usize,
        delta: i16,
    ) -> Result<(), InterpreterError> {
        if let Some(ref mut floor_value) = self.cells[index] {
            *floor_value += delta;
            self.hand = Some(*floor_value);
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::EmptyFloorValue,
                location: command.location,
            });
        }
        self.program_cursor += 1;
        self.step_counter += 1;
        Ok(())
    }
    fn eval_bump_plus(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        self.eval_bump(command, index, 1)
    }
    fn eval_bump_minus(&mut self, command: &Token, index: usize) -> Result<(), InterpreterError> {
        self.eval_bump(command, index, -1)
    }

    fn eval_jump(&mut self, command: &Token, label: usize) -> Result<(), InterpreterError> {
        if let Some(line) = self.jump_table.get(&label) {
            self.program_cursor = *line;
        } else {
            return Err(InterpreterError {
                value: InterpreterErrorKind::UnexistedJumpTarget,
                location: command.location,
            });
        }
        self.step_counter += 1;
        Ok(())
    }
    fn eval_jump_if_zero(&mut self, command: &Token, label: usize) -> Result<(), InterpreterError> {
        if self.hand == Some(0) {
            if let Some(line) = self.jump_table.get(&label) {
                self.program_cursor = *line;
            } else {
                return Err(InterpreterError {
                    value: InterpreterErrorKind::UnexistedJumpTarget,
                    location: command.location,
                });
            }
        } else {
            self.program_cursor += 1;
        }
        self.step_counter += 1;
        Ok(())
    }
    fn eval_jump_if_neg(&mut self, command: &Token, label: usize) -> Result<(), InterpreterError> {
        if let Some(value) = self.hand {
            if value < 0 {
                if let Some(line) = self.jump_table.get(&label) {
                    self.program_cursor = *line;
                } else {
                    return Err(InterpreterError {
                        value: InterpreterErrorKind::UnexistedJumpTarget,
                        location: command.location,
                    });
                }
            } else {
                self.program_cursor += 1;
            }
        } else {
            self.program_cursor += 1;
        }
        self.step_counter += 1;
        Ok(())
    }
    fn init(&mut self) -> Result<usize, InterpreterError> {
        self.cells = vec![];
        for _ in 0..6 {
            self.cells.push(None);
        }
        let mut jump_targets = HashMap::new();
        let mut jump_table = HashMap::new();
        for i in 0..self.program.len() {
            match &self.program[i].value {
                TokenKind::JumpTarget(label) => {
                    jump_targets.insert(label, i);
                }
                TokenKind::Jump(label)
                | TokenKind::JumpIfZero(label)
                | TokenKind::JumpIfNeg(label) => {
                    jump_table.insert(i, label);
                }
                _ => {}
            };
        }
        for (index, label) in jump_table.iter() {
            if jump_targets.contains_key(label) {
                self.jump_table
                    .insert(*index, *jump_targets.get(label).unwrap());
            } else {
                return Err(InterpreterError {
                    value: InterpreterErrorKind::UnexistedJumpTarget,
                    location: self.program[*index].location,
                });
            }
        }

        Ok(0)
    }

    pub fn eval(&mut self, program: &Program) -> Result<usize, InterpreterError> {
        self.program = (*program).clone();
        if let Err(e) = self.init() {
            return Err(e);
        };
        while self.program_cursor < self.program.len() {
            let command = &self.program[self.program_cursor].clone();
            trace!("step:{}\tcommand:{:?}", self.step_counter, command);
            let res = match command.value {
                TokenKind::InBox => self.eval_inbox(command),
                TokenKind::OutBox => self.eval_outbox(command),
                TokenKind::CopyFrom(index) => self.eval_copy_from(command, index),
                TokenKind::CopyTo(index) => self.eval_copy_to(command, index),
                TokenKind::Add(index) => self.eval_add(command, index),
                TokenKind::Sub(index) => self.eval_sub(command, index),
                TokenKind::BumpPlus(index) => self.eval_bump_plus(command, index),
                TokenKind::BumpMinus(index) => self.eval_bump_minus(command, index),
                TokenKind::Jump(_) => self.eval_jump(command, self.program_cursor),
                TokenKind::JumpIfZero(_) => self.eval_jump_if_zero(command, self.program_cursor),
                TokenKind::JumpIfNeg(_) => self.eval_jump_if_neg(command, self.program_cursor),
                _ => {
                    self.program_cursor += 1;
                    Ok(())
                }
            };
            if res.is_err() {
                let err = res.err().unwrap();
                // if an EmptyInBox error is happened, worker will exit with status 0.
                if err.value == InterpreterErrorKind::EmptyInBox {
                    debug!("EmptyInBox and return.");
                    return Ok(0);
                }
                return Err(err);
            }
        }
        Ok(0)
    }
}
