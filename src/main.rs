mod copy_helpers;
mod compiler;

#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;
use std::io;
use std::io::prelude::*;

use copy_helpers::*;

const PROGRAM: &'static str = "wWWwwww";

pub trait Instruction : InstructionClone + Sync + Debug {
    fn eval(&self, ced: &mut CED);
}

#[derive(Clone, Debug)]
struct Application {
    fun: usize,
    arg: usize,
}

impl Instruction for Application {
    fn eval(&self, ced: &mut CED) {
        let fun_val = ced.get_env_val(self.fun).clone();
        let arg_val = ced.get_env_val(self.arg).clone();

        fun_val.apply(ced, &arg_val);
    }
}

#[derive(Clone, Debug)]
struct Abstraction {
    arity: usize,
    body: Vec<Box<dyn Instruction>>,
}

impl Instruction for Abstraction {
    fn eval(&self, ced: &mut CED) {
        if self.arity == 1 {
            ced.push_env_code(self.body.clone());
        } else {
            let abs = Abstraction{arity: self.arity-1, body: self.body.clone()};
            ced.push_env_code(vec!(Box::new(abs)));
        }
    }
}

pub trait Value : ValueClone + Sync + Debug {
    fn get_char(&self) -> Option<u8> {
        None
    }

    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>);
}


#[derive(Clone, Debug)]
struct CE {
    code: Vec<Box<dyn Instruction>>,
    env: Vec<Box<dyn Value>>,
}

impl Value for CE {
    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>) {
        ced.save_dump(self.clone());
        ced.push_env(val.clone());
    }
}


lazy_static! {
    static ref CE_TRUE : CE = CE{
        code: vec!(Box::new(
            Abstraction{
                arity: 1, 
                body: vec!(Box::new(
                    Application {fun: 2, arg: 3}
                ))
            })), 
        env: Vec::new()
    };

    static ref CE_FALSE : CE = CE{
        code: vec!(Box::new(
            Abstraction{
                arity: 1,
                body: Vec::new()
            })), 
        env: Vec::new()
    };
}


// PRIMITIVES
#[derive(Clone, Debug)]
struct CharFn {
    char: u8
}

impl CharFn {
    fn equals(&self, val: &Box<dyn Value>) -> bool {
        match val.get_char() {
            None => false,
            Some(c) => self.char == c
        }
    }
}

impl Value for CharFn {
    fn get_char(&self) -> Option<u8> {
        Some(self.char)
    }

    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>) {
        if self.equals(val) {
            ced.push_env(Box::new(CE_TRUE.clone()));
        } else {
            ced.push_env(Box::new(CE_FALSE.clone()));
        }
    }
}

#[derive(Clone, Debug)]
struct OutFn {
    
}

impl Value for OutFn {
    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>) {
        print!("{}", val.get_char().unwrap() as char);
        ced.push_env(val.clone());
    }
}

#[derive(Clone, Debug)]
struct InFn {

}

impl Value for InFn {
    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>) {
        let char = io::stdin().bytes().next();
        ced.push_env(
            match char {
                None | Some(Err(_)) => {
                    println!("Read failed!");
                    val.clone()
                },
                Some(Ok(c)) => {
                    println!("Read {}", c);
                    Box::new(CharFn {char: c})
                },
        });
    }
}

#[derive(Clone, Debug)]
struct SuccFn {

}

impl Value for SuccFn {
    fn apply(&self, ced: &mut CED, val: &Box<dyn Value>) {
        let c = val.get_char().unwrap();
        let c_inc = if c == 255 { 0 } else { c + 1 };
        ced.push_env(Box::new(CharFn{char: c_inc}));
    }
}


#[derive(Debug)]
pub struct CED {
    code: Vec<Box<dyn Instruction>>,
    env: Vec<Box<dyn Value>>,
    dumps: Vec<CE>,
}

impl CED {
    fn push_env(&mut self, v: Box<dyn Value>) {
        self.env.push(v);
    }

    fn get_env_val(&self, idx: usize) -> &Box<dyn Value> {
        &self.env[self.env.len() - idx]
    }

    fn push_env_code(&mut self, c: Vec<Box<dyn Instruction>>) {
        let ce = CE{code: c, env: self.env.clone()};
        self.push_env(Box::new(ce));
    }

    fn pop_code(&mut self) -> Option<Box<dyn Instruction>> {
        self.code.pop()
    }

    fn save_dump(&mut self, new_ce : CE) {
        let code = std::mem::replace(&mut self.code, new_ce.code.clone());
        let env = std::mem::replace(&mut self.env, new_ce.env.clone());
        let dump_ce = CE{code: code, env: env };
        self.dumps.push(dump_ce); 
    }

    fn restore_dump(&mut self) -> bool {
        let dump = self.dumps.pop();
        if let Some(dump_ce) = dump {
            self.code = dump_ce.code;
            let f = self.env.last().unwrap().clone();
            self.env = dump_ce.env;
            self.push_env(f);
            true
        } else {
            false 
        }
    }
}

fn main() {
    let mut ced = compiler::compile(PROGRAM);
    loop {
        if let Some(insn) = ced.pop_code() {
            insn.eval(&mut ced); 
        } else {
            if !ced.restore_dump() {
                break;
            }
        }
    }
}

