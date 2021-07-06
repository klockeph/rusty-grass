#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;
use std::io;
use std::io::prelude::*;


const PROGRAM: &'static str = "wWWwwww";

trait Instruction : InstructionClone + Sync + Debug {
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

trait Value : ValueClone + Sync + Debug {
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
struct CED {
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

#[derive(PartialEq, Debug)]
enum Token {
    UpperW,
    LowerW,
    LowerV
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut found_w = false;
    for c in s.chars() {
        let t = match c {
                'w' => Token::LowerW,
                'W' => Token::UpperW,
                'v' => Token::LowerV,
                 _  => continue
        };

        found_w |= t == Token::LowerW;

        if found_w {
            tokens.push(t);
        }
    }
    tokens
}


fn split(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut split = Vec::new();
    let mut current = Vec::new();

    for t in tokens.into_iter() {
        match t {
            Token::LowerV => {
                if current.len() > 0  {
                    split.push(current); 
                }
                current = Vec::new() },
            c => current.push(c)
        }
    }
    if current.len() > 0 {
        split.push(current);
    }
    split
}

fn parse_abstraction(ts: &[Token]) -> Vec<Box<dyn Instruction>> {
    let mut arg_num = 0;
    let mut body = Vec::new();
    for i in 0..ts.len() {
        match ts[i] {
            Token::LowerW => arg_num += 1,
            Token::UpperW => {
                body = parse_application(&ts[i..]);
                break;
            },
            _ => panic!("")
        }
    }
    vec!(Box::new(Abstraction { arity: arg_num, body: body }))
}

fn parse_application(ts: &[Token]) -> Vec<Box<dyn Instruction>> {
    let mut apps : Vec<Box<dyn Instruction>> = Vec::new();

    let mut fun = 0;
    let mut arg = 0;

    for t in ts {
        match t {
            Token::LowerW => arg += 1, 
            Token::UpperW => {
                if arg == 0 {
                    fun += 1;
                } else {
                    apps.push(Box::new(Application{fun: fun, arg: arg}));
                    fun = 1;
                    arg = 0;
                }
            }
            _ => panic!("")
        }
    }

    if fun > 0 && arg > 0 {
        apps.push(Box::new(Application{fun: fun, arg: arg}));
    }
    apps 
}


fn parse(s: &str) -> CED {
    let tokens = tokenize(s);
    let splits = split(tokens);

    let mut code = Vec::new();

    for s in splits.into_iter() {
        code.append( &mut 
            match s[0] {
                Token::LowerW => parse_abstraction(&s),
                Token::UpperW => parse_application(&s),
                _ => panic!("v remaining in splits??")
        });
    }

    let env : Vec<Box<dyn Value>> = vec!(
        Box::new(InFn{}),
        Box::new(CharFn{char: 'w' as u8}),
        Box::new(SuccFn{}),
        Box::new(OutFn{}),
    );

    let dump_code = vec!(Box::new(Application {arg: 1, fun: 1}) as Box<dyn Instruction>);
    let dump_env = Vec::new(); 
    let dump_ce = CE { code: dump_code, env: dump_env };

    CED { code: code, env: env, dumps: vec!(dump_ce) }
}

fn main() {
    let mut ced = parse(PROGRAM);
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



// I want to hide this crap
trait ValueClone {
    fn clone_box(&self) -> Box<dyn Value>;
}

impl<T> ValueClone for T 
where
    T: 'static + Value + Clone
{
    fn clone_box(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Value> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

trait InstructionClone {
    fn clone_box(&self) -> Box<dyn Instruction>;
}

impl<T> InstructionClone for T
where
    T: 'static + Instruction + Clone
{
    fn clone_box(&self) -> Box<dyn Instruction> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Instruction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

