#![recursion_limit="48"]
// #![feature(trace_macros)]
// trace_macros!(true);

use std::rc::Rc;


macro_rules! noun {
  () => {};
  ($it:literal) => { Rc::new(Noun::Atom($it)) };
  ($it:literal $($rest:tt)*) => { Rc::new(Noun::Cell((noun!($it), noun!($($rest)*)))) };
  ([$($sub:tt)*]) => { noun!($($sub)*) };
  ([$($sub:tt)*] $($rest:tt)*) => { Rc::new(Noun::Cell((noun!($($sub)*), noun!($($rest)*)))) };
  (($($sub:tt)*) $($rest:tt)*) => { noun!([$($sub)*] $($rest)*) };
  ({$($sub:tt)*} $($rest:tt)*) => { noun!([$($sub)*] $($rest)*) };
}


#[derive(Debug)]
enum Noun {
  Atom(u128),
  Cell((Rc<Noun>,Rc<Noun>)),
}
use Noun::*;

impl Noun {
  fn to_string(&self) -> String {
    match self {
      Atom(a) => format!("{}", a),
      Cell((ref l, ref r)) => format!("[{} {}]", l.to_string(), r.to_string()),
    }
  }
}

fn main() {
  // 0, slot
  // *[a 0 b]            /[b a]
  // [[3 4 5] 0 7] -> 5
  // let n = noun![[3 4 5] 0 7];

  // 1, constant
  // *[a 1 b]            b
  // [4 [1 8]] -> 8
  // let n = noun![4 1 8];

  // 2, evaluate
  // *[a 2 b c]          *[*[a b] *[a c]]
  // *[[8 [4 0 1]] 2 [[0 2] [0 3]]]     =>     *[8 [4 0 1]]    =>    9
  let n = noun![[8 [4 0 1]] 2 [[0 2] [0 3]]];

  // 3, depth
  // *[a 3 b]            ?*[a b]
  // [0 3 0 1] -> 1
  // [[0 0] 3 0 1] -> 0
  // let n = noun![0 3 0 1];
  // let n = noun![[0 0] 3 0 1];

  // 4, inc
  // [9 4 0 1] -> 10
  // [[0 0] 4 0 1] -> X
  // *[a 4 b]            +*[a b]
  // let n = noun![9 4 0 1];
  // let n = noun![[0 0] 4 0 1];


  // *[a 5 b c]          =[*[a b] *[a c]]
  // *[a 6 b c d]        *[a *[[c d] 0 *[[2 3] 0 *[a 4 4 b]]]]
  // *[a 7 b c]          *[*[a b] c]
  // *[a 8 b c]          *[[*[a b] a] c]
  // *[a 9 b c]          *[*[a c] 2 [0 1] 0 b]
  // *[a 10 [b c] d]     #[b *[a c] *[a d]]
  // *[a 11 [b c] d]     *[[*[a c] *[a d]] 0 3]
  // *[a 11 b c]         *[a c]

  println!("{} ->  {:?}", n.to_string(), nock(&n).map(|n| n.to_string()));

}

type EvalResult = Result<Rc<Noun>, &'static str>;

fn nock(n: &Rc<Noun>) -> EvalResult {
  match **n {
    Atom(_) => Err("attempt to evaluate atom"),
    Cell((ref subject, ref formula)) => apply(subject, formula)
  }
}

fn apply(subject: &Rc<Noun>, formula: &Rc<Noun>) -> EvalResult {
  match **formula {
    Atom(_) => Err("attempt to apply atom as a formula"),
    Cell((ref head, ref tail)) => {
      match **head {
        Atom(n) => op(n, subject, tail),
        Cell(_) => Err("autocons not implemented"),
      }
    },
  }
}

fn op(instruction: u128, subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
  match instruction {
    0 => slot(args, subject),
    1 => Ok(args.clone()),
    3 => apply(subject, args).map(|n| depth(&n)),
    4 => apply(subject, args).and_then(inc),
    _ => Err("unimplemented opcode"),
  }
}

fn depth(n: &Rc<Noun>) -> Rc<Noun> {
  match **n {
    Atom(_) => Rc::new(Atom(1)),
    Cell(_) => Rc::new(Atom(0)),
  }
}

fn inc(n: Rc<Noun>) -> EvalResult {
  match *n {
    Atom(m) => Ok(Rc::new(Atom(m+1))),
    Cell(_) => Err("attempt to increment a cell"),
  }
}

fn slot(address: &Rc<Noun>, subject: &Rc<Noun>) -> EvalResult {
  match **address {
    Cell(_) => Err("slot addresses must be atoms!"),
    Atom(0) => Err("slot address can't be zero!"),
    Atom(n) => slot_n(n, subject),
  }
}

fn slot_n(address: u128, subject: &Rc<Noun>) -> EvalResult {
  if address == 1 {
    return Ok(subject.clone());
  }
  match **subject {
    Atom(_) => Err("nock 0 error - attempt to address through an atom"),
    Cell((ref l, ref r)) => {
      if address%2 == 0 {
        slot_n(address / 2, l)
      } else {
        slot_n(address / 2, r)
      }
    }
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

