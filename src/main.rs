#![recursion_limit="48"]
// #![feature(trace_macros)]
// trace_macros!(true);

use std::rc::Rc;


macro_rules! noun {
    () => {};
    ($it:literal) => { Rc::new(Noun::Atom($it)) };
    ($it:literal $($rest:tt)*) => { Rc::new(Noun::Cell(noun!($it), noun!($($rest)*))) };
    ($it:ident) => { $it.clone() };
    ($it:ident $($rest:tt)*) => { Rc::new(Noun::Cell($it.clone(), noun!($($rest)*))) };
    (@$it:expr) => { $it };
    (@$it:expr, $($rest:tt)*) => { Rc::new(Noun::Cell($it, noun!($($rest)*))) };
    ([$($sub:tt)*]) => { noun!($($sub)*) };
    ([$($sub:tt)*] $($rest:tt)*) => { Rc::new(Noun::Cell(noun!($($sub)*), noun!($($rest)*))) };
    (($($sub:tt)*) $($rest:tt)*) => { noun!([$($sub)*] $($rest)*) };
    ({$($sub:tt)*} $($rest:tt)*) => { noun!([$($sub)*] $($rest)*) };
}


#[derive(Debug)]
enum Noun {
    Atom(u128),
    Cell(Rc<Noun>,Rc<Noun>),
}
use Noun::*;

impl Noun {
    fn to_string(&self) -> String {
        match self {
            Atom(a) => format!("{}", a),
            Cell(ref l, ref r) => format!("[{} {}]", l.to_string(), r.to_string()),
        }
    }

    fn open(&self) -> Result<(&Rc<Noun>,&Rc<Noun>), &'static str> {
        match self {
            Atom(_) => Err("expecting cell, got atom"),
            Cell(ref l, ref r) => Ok((l, r)),
        }
    }

    fn open_or(&self, s: &'static str) -> Result<(&Rc<Noun>,&Rc<Noun>), &'static str> {
        match self {
            Atom(_) => Err(s),
            Cell(ref l, ref r) => Ok((l, r)),
        }
    }
}

fn main() {
    let mut v = Vec::new();
    // 0, slot
    // *[a 0 b]            /[b a]
    // [[3 4 5] 0 7] -> 5
    v.push((noun![[3 4 5] 0 7], Ok("5")));

    // 1, constant
    // *[a 1 b]            b
    // [4 [1 8]] -> 8
    v.push((noun![4 1 8], Ok("8")));
    
    // autocons
    let result = noun![4 2].to_string();
    v.push((noun![0 [[1 4] [1 2]]], Ok(&result)));

    // 2, evaluate
    // *[a 2 b c]          *[*[a b] *[a c]]
    // *[[8 [4 0 1]] 2 [[0 2] [0 3]]]     =>     *[8 [4 0 1]]    =>    9
    v.push((noun![[8 [4 0 1]] 2 [[0 2] [0 3]]], Ok("9")));

    // 3, depth
    // *[a 3 b]            ?*[a b]
    // [0 3 0 1] -> 1
    // [[0 0] 3 0 1] -> 0
    v.push((noun![0 3 0 1], Ok("1")));
    v.push((noun![[0 0] 3 0 1], Ok("0")));

    // 4, inc
    // [9 4 0 1] -> 10
    // [[0 0] 4 0 1] -> X
    // *[a 4 b]            +*[a b]
    v.push((noun![2 4 0 1], Ok("3")));
    v.push((noun![[0 0] 4 0 1], Err("nah")));

    // 5, equality
    // *[a 5 b c]          =[*[a b] *[a c]]
    // [[0 0] 5 [0 2] [0 1]]  (0 == [0 0])? => false => 1
    // [0 5 [0 1] [0 1]]  (0 == 0)? => true => 0
    // [[0 1] 5 [0 2] [0 3]]  (0 == 1)? => false => 1
    // [[0 0] 5 [0 1] [0 1]]  ([0 0] == [0 0])? => true => 0
    // [[[0 0] [0 1]] 5 [0 2] [0 3]]  ([0 0] == [0 1])? => false => 1
    v.push((noun![[0 0] 5 [0 2] [0 1]], Ok("1")));         // 1
    v.push((noun![0 5 [0 1] [0 1]], Ok("0")));             // 0
    v.push((noun![[0 1] 5 [0 2] [0 3]], Ok("1")));         // 1
    v.push((noun![[0 0] 5 [0 1] [0 1]], Ok("0")));         // 0
    v.push((noun![[[0 0] [0 1]] 5 [0 2] [0 3]], Ok("1"))); // 1

    // 10, edit
    // *[a 10 [b c] d]     #[b *[a c] *[a d]]

    // 6, if/then/else
    // *[a 6 b c d]        *[a *[[c d] 0 *[[2 3] 0 *[a 4 4 b]]]]
    // [[0 [4 5]] 6 [0 2] [0 5] [0 7]]  => 4
    // [[1 [4 5]] 6 [0 2] [0 5] [0 7]]  => 5
    v.push((noun![[0 [4 5]] 6 [0 2] [0 5] [0 7]], Ok("4")));
    v.push((noun![[1 [4 5]] 6 [0 2] [0 5] [0 7]], Ok("5")));

    // 7, compose
    // *[a 7 b c]          *[*[a b] c]
    v.push((noun![1 7 [4 0 1] [4 0 1]], Ok("3")));

    // 8, push
    // *[a 8 b c]          *[[*[a b] a] c]
    let result = noun![0 1].to_string();
    v.push((noun![0 8 [4 0 1] [[0 3] [0 2]]], Ok(&result)));

    // 9, invoke
    // *[a 9 b c]          *[*[a c] 2 [0 1] 0 b]
    v.push((noun![[1 5] 9 1 0 1], Ok("5")));
    
    
    // *[a 11 [b c] d]     *[[*[a c] *[a d]] 0 3]
    // *[a 11 b c]         *[a c]


    for (n, expected) in v {
        println!("{} ->  {:?}/{:?}", n.to_string(), nock(&n).map(|n| n.to_string()), expected);
    }

}

type EvalResult = Result<Rc<Noun>, &'static str>;

fn nock(n: &Rc<Noun>) -> EvalResult {
    let (subject, formula) = n.open_or("attempt to evaluate atom")?;
    apply(subject, formula)
}

fn apply(subject: &Rc<Noun>, formula: &Rc<Noun>) -> EvalResult {
    let (head, tail) = formula.open_or("attempt to apply atom as a formula")?;
    match **head {
        Atom(n) => op(n, subject, tail), // run instruction
        Cell(_,_) => {                   // autocons
            let left = apply(subject, head)?;
            let right = apply(subject, tail)?;
            Ok(Rc::new(Cell(left, right)))
        },
    }
}

fn op(instruction: u128, subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    match instruction {
        0 => slot(args, subject),
        1 => Ok(args.clone()),
        2 => eval(subject, args),
        3 => apply(subject, args).map(|n| depth(&n)),
        4 => apply(subject, args).and_then(inc),
        5 => eq(subject, args),
        6 => macro_six(subject, args),
        7 => macro_seven(subject, args),
        8 => macro_eight(subject, args),
        9 => macro_nine(subject, args),
        _ => Err("unimplemented opcode"),
    }
}

    // *[a 9 b c]          *[*[a c] 2 [0 1] 0 b]
fn macro_nine(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (b, c) = args.open()?;
    let core = apply(subject, c)?;
    apply(&core, &noun![2 [0 1] 0 b])
}

fn macro_eight(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (b, c) = args.open()?;
    let var = apply(subject, b)?;
    apply(&noun![var subject], c)
}

fn macro_seven(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (b, c) = args.open()?;
    let new_subject = apply(subject, b)?;
    apply(&new_subject, c)
}

fn macro_six(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (b, rest) = args.open()?;
    let (c, d) = rest.open()?;
    let predicate = nock(&noun![subject 4 4 b])?;
    let branch = nock(&noun![[2 3] 0 predicate])?;
    let formula = nock(&noun![[c d] 0 branch])?;
    apply(subject, &formula)
}

fn eq(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (get_left, get_right) = args.open()?;
    let left = apply(subject, get_left)?;
    let right = apply(subject, get_right)?;
    if noun_eq(&left, &right) {
        Ok(Rc::new(Atom(0)))
    } else {
        Ok(Rc::new(Atom(1)))
    }
}

fn noun_eq(a: &Rc<Noun>, b: &Rc<Noun>) -> bool {
    match (&**a,&**b) {
        (Atom(a), Atom(b)) => a == b,
        (Cell(ref ll, ref lr), Cell(ref rl, ref rr)) => {
            noun_eq(ll, rl) && noun_eq(lr, rr)
        },
        _ => false,
    }
}

fn eval(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (get_subject, get_formula) = args.open_or("can't apply atom to a subject")?;
    let new_subject = apply(subject, get_subject)?;
    let new_formula = apply(subject, get_formula)?;
    apply(&new_subject, &new_formula)
}

fn depth(n: &Rc<Noun>) -> Rc<Noun> {
    match **n {
        Atom(_) => Rc::new(Atom(1)),
        Cell(_,_) => Rc::new(Atom(0)),
    }
}

fn inc(n: Rc<Noun>) -> EvalResult {
    match *n {
        Atom(m) => Ok(Rc::new(Atom(m+1))),
        Cell(_,_) => Err("attempt to increment a cell"),
    }
}

fn slot(address: &Rc<Noun>, subject: &Rc<Noun>) -> EvalResult {
    match **address {
        Cell(_,_) => Err("slot addresses must be atoms!"),
        Atom(0) => Err("slot address can't be zero!"),
        Atom(n) => slot_n(n, subject),
    }
}

fn slot_n(address: u128, subject: &Rc<Noun>) -> EvalResult {
    if address == 1 {
        return Ok(subject.clone());
    }

    let (l, r) = subject.open_or("nock 0 error - attempt to address through an atom")?;
    if address % 2 == 0 {
        slot_n(address / 2, l)
    } else {
        slot_n(address / 2, r)
    }
}


/*
   Nock 4K
   A noun is an atom or a cell.  An atom is a natural number.  A cell is an ordered pair of nouns.
   Reduce by the first matching pattern; variables match any noun.

 *[a [b c] d]        [*[a b c] *[a d]]

 *[a 0 b]            /[b a]
 *[a 1 b]            b
 *[a 2 b c]          *[*[a b] *[a c]]
 *[a 3 b]            ?*[a b]
 *[a 4 b]            +*[a b]
 *[a 5 b c]          =[*[a b] *[a c]]

 *[a 6 b c d]        *[a *[[c d] 0 *[[2 3] 0 *[a 4 4 b]]]]
 *[a 7 b c]          *[*[a b] c]
 *[a 8 b c]          *[[*[a b] a] c]
 *[a 9 b c]          *[*[a c] 2 [0 1] 0 b]
 *[a 10 [b c] d]     #[b *[a c] *[a d]]

 *[a 11 [b c] d]     *[[*[a c] *[a d]] 0 3]
 *[a 11 b c]         *[a c]

 *a                  *a

 nock(a)             *a
 [a b c]             [a [b c]]

 ?[a b]              0
 ?a                  1
 +[a b]              +[a b]
 +a                  1 + a
 =[a a]              0
 =[a b]              1

 /[1 a]              a
 /[2 a b]            a
 /[3 a b]            b
 /[(a + a) b]        /[2 /[a b]]
 /[(a + a + 1) b]    /[3 /[a b]]
 /a                  /a

#[1 a b]            a
#[(a + a) b c]      #[a [b /[(a + a + 1) c]] c]
#[(a + a + 1) b c]  #[a [/[(a + a) c] b] c]
#a                  #a

*/

