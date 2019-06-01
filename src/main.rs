use std::rc::Rc;

#[derive(Debug)]
enum Noun {
  Atom(u128),
  Cell((Rc<Noun>,Rc<Noun>)),
}
use Noun::*;

fn main() {
    let right = Cell((Rc::new(Atom(1)), Rc::new(Atom(8))));
    let constfn = Cell((Rc::new(Atom(4)), Rc::new(right)));
    dbg!(nock(&constfn));
    dbg!(&constfn);
    // dbg!(nock(&right));
}

fn nock(n: &Noun) -> Result<&Noun, &'static str> {
    match n {
        Atom(_) => Err("attempt to evaluate atom"),
        Cell((subject,formula)) => {
            match **formula {
                Atom(_) => Err("attempt to apply atom as formula"),
                Cell((ref l, ref r)) => apply(subject, l, r),
            }
        },
    }
}

fn apply<'a>(subject: &'a Noun, head: &'a Noun, tail: &'a Noun) -> Result<&'a Noun, &'static str> {
    match head {
      Cell(_) => Err("autocons not implemented"),
      Atom(1) => Ok(tail),
      Atom(_) => Err("unimplemented opcode"),
    }
}


/*
Nock 4K
A noun is an atom or a cell.  An atom is a natural number.  A cell is an ordered pair of nouns.
Reduce by the first matching pattern; variables match any noun.

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

*/
