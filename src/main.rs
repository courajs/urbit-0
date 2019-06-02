#![recursion_limit="48"]
// #![feature(trace_macros)]
// trace_macros!(true);

use std::rc::Rc;


macro_rules! noun {
    () => {};
    ($it:literal) => { Rc::new(Noun::Atom($it)) };
    ($it:literal $($rest:tt)*) => { Rc::new(Noun::Cell((noun!($it), noun!($($rest)*)))) };
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
  // [4 [1 8]] -> 8
  // println!("{}
  // dbg!(nock(noun![4 1 8])).unwrap();

  let n = noun![[9 9] 0 2];
  println!("{} ->  {:?}", n.to_string(), nock(n).map(|n| n.to_string()));
  // dbg!(nock(noun![[9 9] 0 2]).map(|n| n.to_string()));
  // dbg!(format!("{:?}", nock(noun![[9 9] 0 1]).unwrap()));

}

fn nock(n: Rc<Noun>) -> Result<Rc<Noun>, &'static str> {
    match *n {
        Atom(_) => Err("attempt to evaluate atom"),
        Cell((ref subject, ref formula)) => {
            match **formula {
                Atom(_) => Err("attempt to apply atom as formula"),
                Cell((ref l, ref r)) => apply(subject.clone(), l.clone(), r.clone()),
            }
        },
    }
}

fn apply(subject: Rc<Noun>, head: Rc<Noun>, tail: Rc<Noun>) -> Result<Rc<Noun>, &'static str> {
    match *head {
      Cell(_) => Err("autocons not implemented"),
      Atom(0) => slot(tail, subject),
      Atom(1) => Ok(tail),
      Atom(_) => Err("unimplemented opcode"),
    }
}

fn slot(address: Rc<Noun>, subject: Rc<Noun>) -> Result<Rc<Noun>, &'static str> {
  match *address {
    Cell(_) => Err("slot addresses must be atoms!"),
    Atom(0) => Err("slot address can't be zero!"),
    Atom(1) => Ok(subject),
    Atom(2) => {
      match *subject {
        Atom(_) => Err("address too deep -- cannot address past an atom"),
        Cell((ref l,_)) => Ok(l.clone()),
      }
    },
    Atom(_) => Err("not yet implemented"),
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

