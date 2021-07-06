use crate::*;

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

fn parse_abstraction(ts: &[Token]) -> VecDeque<Box<dyn Instruction>> {
    let mut arg_num = 0;
    let mut body = VecDeque::new();
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
    let mut v = VecDeque::new();
    v.push_back(Box::new(Abstraction { arity: arg_num, body: body }) as Box<dyn Instruction>);
    v
}

fn parse_application(ts: &[Token]) -> VecDeque<Box<dyn Instruction>> {
    let mut apps : VecDeque<Box<dyn Instruction>> = VecDeque::new();

    let mut fun = 0;
    let mut arg = 0;

    for t in ts {
        match t {
            Token::LowerW => arg += 1, 
            Token::UpperW => {
                if arg == 0 {
                    fun += 1;
                } else {
                    apps.push_back(Box::new(Application{fun: fun, arg: arg}));
                    fun = 1;
                    arg = 0;
                }
            }
            _ => panic!("")
        }
    }

    if fun > 0 && arg > 0 {
        apps.push_back(Box::new(Application{fun: fun, arg: arg}));
    }
    apps 
}


pub fn compile(s: &str) -> CED {
    let tokens = tokenize(s);
    let splits = split(tokens);

    let mut code = VecDeque::new();

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

    let mut dump_code = VecDeque::new();
    dump_code.push_back(Box::new(Application {arg: 1, fun: 1}) as Box<dyn Instruction>);
    let dump_env = Vec::new(); 
    let dump_ce = CE { code: dump_code, env: dump_env };

    CED { code: code, env: env, dumps: vec!(dump_ce) }
}
