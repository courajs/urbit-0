#![recursion_limit="1024"]
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


#[derive(Debug,PartialEq)]
enum Noun {
    Atom(u128),
    Cell(Rc<Noun>,Rc<Noun>),
}
use Noun::*;

impl Noun {
    fn value(&self) -> Result<u128, &'static str> {
        match self {
            Atom(n) => Ok(*n),
            Cell(_,_) => Err("Expecting atom, got cell"),
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
        10 => edit(subject, args),
        _ => Err("unimplemented opcode"),
    }
}

// 10, edit
// *[a 10 [b c] d]     #[b *[a c] *[a d]]
fn edit(subject: &Rc<Noun>, args: &Rc<Noun>) -> EvalResult {
    let (head, d) = args.open()?;
    let (b, c) = head.open()?;
    let address = b.value()?;
    let replacement = apply(subject, c)?;
    let base = apply(subject, d)?;
    edit_n(address, &replacement, &base)
}

//                  1
//          2              3
//      4       5       6      7
//    8   9   10 11   12 13  14 15

//   [[[8 9] [10 11]] [[12 13] [14 15]]]
//   #[1 b c]            b
//   #[(a + a) b c]      #[a [b /[(a + a + 1) c]] c]
//   #[(a + a + 1) b c]  #[a [/[(a + a) c] b] c]
fn edit_n(address: u128, replacement: &Rc<Noun>, base: &Rc<Noun>) -> EvalResult {
    if address == 1 {
        return Ok(replacement.clone());
    }

    if address % 2 == 0 {
        edit_n(address / 2, &Rc::new(Cell(replacement.clone(), slot_n(address+1, base)?)), base)
    } else {
        edit_n(address / 2, &Rc::new(Cell(slot_n(address-1, base)?, replacement.clone())), base)
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
        Atom(n) => {
            slot_n(n, subject)
        },
    }
}

fn slot_n(address: u128, subject: &Rc<Noun>) -> EvalResult {
    if address == 1 {
        return Ok(subject.clone());
    }

    let new_base = slot_n(address / 2, subject)?;
    let (l, r) = new_base.open_or("nock 0 error - attempt to address through an atom")?;
    if address % 2 == 0 {
        Ok(l.clone())
    } else {
        Ok(r.clone())
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



#[cfg(test)]
mod tests {
    use super::*;

    // 0, slot
    // *[a 0 b]            /[b a]
    #[test]
    fn nock_0() {
        assert_eq!(
            nock(&noun! [[3 4 5] 0 7]  ),
               Ok(noun! [5]            )
        );
        assert_eq!(
            apply(&noun![531 25 99], &noun![0 6]),
            Ok(noun![25])
        );
    }

    // 1, constant
    // *[a 1 b]            b
    #[test]
    fn nock_1() {
        assert_eq!(
            nock(&noun! [4 1 8]  ),
               Ok(noun! [8]      )
        );
    }

    #[test]
    fn autocons() {
        assert_eq!(
            nock(&noun! [0 [1 4] 1 2]  ),
               Ok(noun! [4 2]          )
        );
    }

    // 2, evaluate
    // *[a 2 b c]          *[*[a b] *[a c]]
    #[test]
    fn nock_2() {
        assert_eq!(
            nock(&noun! [[4 0 1] 2 [[0 2] [0 1]]]  ),
               Ok(noun! [5])
        );
    }

    // 3, depth
    // *[a 3 b]            ?*[a b]
    #[test]
    fn nock_3() {
        assert_eq!(
            nock(&noun! [0 3 0 1]  ),
               Ok(noun! [1])
        );
        assert_eq!(
            nock(&noun! [[0 0] 3 0 1]  ),
               Ok(noun! [0])
        );
    }

    // 4, inc
    // *[a 4 b]            +*[a b]
    #[test]
    fn nock_4() {
        assert_eq!(
            nock(&noun! [2 4 0 1]  ),
               Ok(noun! [3])
        );
        assert!(nock(&noun! [[0 0] 4 0 1]).is_err());
    }
    
    // 5, equality
    // *[a 5 b c]          =[*[a b] *[a c]]
    #[test]
    fn nock_5() {
        // Any cell != any atom
        // 0 != [0 0]
        // [[0 0] 5 [0 2] [0 1]]
        assert_eq!(
            nock(&noun! [[0 0] 5 [0 2] [0 1]]  ),
               Ok(noun! [1])
        );
        // Atoms are equal if they are the same number
        // 0 == 0
        // [0 5 [0 1] [0 1]] 
        assert_eq!(
            nock(&noun! [0 5 [0 1] [0 1]]  ),
               Ok(noun! [0])
        );
        // Atoms are unequal if they are different numbers
        // 0 != 1
        // [[0 1] 5 [0 2] [0 3]]
        assert_eq!(
            nock(&noun! [[0 1] 5 [0 2] [0 3]]  ),
               Ok(noun! [1])
        );
        // Cells are equal if their lefts are equal and their rights are equal
        // [0 0] == [0 0]
        // [[0 0] 5 [0 1] [0 1]]
        assert_eq!(
            nock(&noun! [[0 0] 5 [0 1] [0 1]]  ),
               Ok(noun! [0])
        );
        // Cells are unequal if their left or right differ
        // [0 0] != [0 1]
        // [[[0 0] [0 1]] 5 [0 2] [0 3]]
        assert_eq!(
            nock(&noun! [[[0 0] [0 1]] 5 [0 2] [0 3]]  ),
               Ok(noun! [1])
        );
    }

    // 6, if/then/else
    // *[a 6 b c d]        *[a *[[c d] 0 *[[2 3] 0 *[a 4 4 b]]]]
    #[test]
    fn nock_6() {
        assert_eq!(
            nock(&noun! [[0 [4 5]] 6 [0 2] [0 6] [0 7]]  ),
               Ok(noun! [4])
        );
        assert_eq!(
            nock(&noun! [[1 [4 5]] 6 [0 2] [0 6] [0 7]]  ),
               Ok(noun! [5])
        );
    }


    // 7, compose
    // *[a 7 b c]          *[*[a b] c]
    #[test]
    fn nock_7() {
        assert_eq!(
            nock(&noun! [1 7 [4 0 1] [4 0 1]]  ),
               Ok(noun! [3])
        );
    }



    // 8, push
    // *[a 8 b c]          *[[*[a b] a] c]
    #[test]
    fn nock_8() {
        assert_eq!(
            nock(&noun! [0 8 [4 0 1] [[0 3] [0 2]]]  ),
               Ok(noun! [0 1])
        );
    }


    // 9, invoke
    // *[a 9 b c]          *[*[a c] 2 [0 1] 0 b]
    #[test]
    fn nock_9() {
        assert_eq!(
            nock(&noun! [[1 5] 9 1 0 1]  ),
               Ok(noun! [5])
        );
    }


    fn hash_to_ten(address: u128, replacement: Rc<Noun>, subject: Rc<Noun>) -> Rc<Noun> {
        noun![[replacement subject] 10 [@Rc::new(Atom(address)), [0 2]] [0 3]]
    }
    // 10, edit
    // *[a 10 [b c] d]     #[b *[a c] *[a d]]
    #[test]
    fn nock_10() {
        assert_eq!(
            nock(&hash_to_ten(2, noun![11], noun![22 33])),
               Ok(noun! [11 33])
        );
        assert_eq!(
            nock(&hash_to_ten(3, noun![11], noun![22 33])),
               Ok(noun! [22 11])
        );
        assert_eq!(
            nock(&hash_to_ten(4, noun![11], noun![[22 33] 44])),
               Ok(noun! [[11 33] 44])
        );
        assert_eq!(
            nock(&hash_to_ten(5, noun![11], noun![[22 33] 44])),
               Ok(noun! [[22 11] 44])
        );
    }

    // https://github.com/martindevans/Rock/blob/1bb8508c2b5255139ace2a91fcc8bdeec8dda811/src/noun.rs#L862
    #[test]
    fn rock_tests() {
        assert_eq!(
            nock(&noun! [1 [2 [1 1] [1 [1 7]]]]  ),
               Ok(noun! [7])
        );
    }
    
    #[test]
    fn dec() {
        let subject = noun![2];
        let dec = noun![8 [1 8 [1 0] 8 [1 6 [5 [0 15] 4 0 6] [0 6] 9 2 10 [6 4 0 6] 0 1] 9 2 0 1] 9 2 0 1];
        assert_eq!(apply(&subject, &dec), Ok(noun![1]));
    }

    // From
    // https://github.com/martindevans/Rock/blob/1bb8508c2b5255139ace2a91fcc8bdeec8dda811/readme.md
    #[test]
    #[ignore]
    fn monster() {
        let n = noun![368 [8 [[[[[[[[[[1 [[[1 [8 [9 513 [0 7]] [6 [5 [0 14] [1 0]] [9 13 [0 1]] [9 4 [[0 4] [0 14] [0 5]]]]]] [1 [6 [5 [9 4 [[0 4] [0 14] [0 5]]] [1 0]] [0 14] [9 13 [[0 2] [0 6] [4 [4 [0 14]]] [0 15]]]]]] [0 1]]] [1 [[[1 [8 [9 257 [0 15]] [8 [9 17 [0 31]] [8 [9 65 [0 63]] [8 [9 129 [0 127]] [6 [9 4 [[0 28] [[[1 3] [0 126]] [0 29]]]] [1 1] [6 [9 2 [[0 4] [[0 126] [0 5]]]] [9 125 [[0 2] [0 6] [0 14] [0 30] [0 62] [0 126] [[0 508] [9 2 [[0 60] [9 2 [[0 60] [0 126] [0 61]]] [0 61]]]] [0 255]]] [1 1]]]]]]]] [1 [6 [9 2 [[0 12] [0 254] [0 13]]] [[0 254] [1 1]] [6 [5 [4 [4 [0 508]]] [0 126]] [1 0] [9 125 [[0 2] [0 6] [0 14] [0 30] [0 62] [0 126] [[4 [0 508]] [9 2 [[0 60] [0 509] [0 61]]]] [0 255]]]]]]] [[[1 2] [1 0]] [0 1]]]]] [1 [[1 [6 [5 [4 [0 14]] [0 6]] [0 14] [9 2 [[0 2] [[0 6] [[4 [0 14]] [0 15]]]]]]] [[1 0] [0 1]]]]] [1 [[1 [6 [5 [0 28] [0 6]] [1 0] [6 [5 [0 29] [0 6]] [1 1] [9 2 [[0 2] [[0 6] [[[4 [4 [0 28]]] [4 [4 [0 29]]]] [0 15]]]]]]]] [[[1 0] [1 1]] [0 1]]]]] [1 [[1 [8 [9 33 [0 7]] [6 [9 4 [[0 4] [0 28] [0 5]]] [6 [9 4 [[0 4] [0 29] [0 5]]] [1 0] [1 1]] [1 1]]]] [0 1]]]] [1 [[[1 [8 [9 9 [0 15]] [8 [9 17 [0 31]] [8 [9 129 [0 63]] [6 [5 [0 62] [1 0]] [1 1] [6 [5 [0 62] [1 1]] [1 1] [6 [5 [0 62] [1 2]] [1 0] [6 [5 [0 62] [1 3]] [1 0] [6 [9 2 [[0 4] [0 62] [0 5]]] [1 1] [9 61 [0 1]]]]]]]]]]] [1 [6 [5 [9 2 [[0 28] [[0 126] [0 29]]]] [0 62]] [1 1] [6 [5 [4 [0 253]] [0 62]] [6 [9 4 [[0 12] [[[9 2 [[0 28] [[0 126] [0 29]]]] [0 62]] [0 13]]]] [1 0] [9 61 [[0 2] [0 6] [0 14] [0 30] [0 62] [[4 [0 252]] [0 253]] [0 127]]]] [6 [9 4 [[0 12] [[[9 2 [[0 28] [[0 126] [0 29]]]] [0 62]] [0 13]]]] [9 61 [[0 2] [0 6] [0 14] [0 30] [0 62] [[1 2] [4 [0 253]]] [0 127]]] [9 61 [[0 2] [0 6] [0 14] [0 30] [0 62] [[4 [0 252]] [0 253]] [0 127]]]]]]]] [[[1 2] [1 2]] [0 1]]]]] [1 [[[1 [6 [5 [0 14] [0 15]] [1 1] [9 5 [[0 2] [[0 6] [[0 6] [0 15]]]]]]] [1 [6 [5 [4 [0 28]] [0 13]] [1 1] [6 [5 [4 [0 29]] [0 12]] [1 0] [9 5 [[0 2] [[0 6] [[[4 [0 28]] [4 [0 29]]] [0 15]]]]]]]]] [[[1 0] [1 0]] [0 1]]]]] [1 [[1 [8 [9 5 [0 15]] [6 [5 [0 29] [0 61]] [0 60] [9 2 [[0 6] [0 14] [[[9 2 [[0 4] [[[0 60] [0 28]] [0 5]]]] [4 [0 61]]] [0 31]]]]]]] [[[1 0] [1 0]] [0 1]]]]] [1 [1 [6 [5 [0 13] [0 14]] [0 12] [9 2 [[0 2] [[4 [0 12]] [0 13]] [[4 [0 14]] [0 15]]]]]] [1 0] [0 1]]] [1 0]] [8 [9 512 [0 2]] [9 4 [[0 4] [0 7] [0 5]]]]]];

        assert_eq!(
            nock(&n),
            Ok(noun! [[19 349] 1])
        );
    }
    
    // *[a 11 [b c] d]     *[[*[a c] *[a d]] 0 3]
    // *[a 11 b c]         *[a c]



}

