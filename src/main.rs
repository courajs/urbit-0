#[derive(Debug)]
enum Noun {
    Atom(u128),
    Cell(Box<(Noun,Noun)>),
}
use Noun::*;

fn main() {
    let right = Cell(Box::new((Atom(1), Atom(8))));
    let constfn = Cell(Box::new((Atom(4), right)));
    dbg!(nock(&constfn));
    // dbg!(nock(&right));
}

fn nock(n: &Noun) -> Result<&Noun, &'static str> {
    match n {
        Atom(v) => Err("attempt to evaluate atom"),
        Cell(b) => {
            match **b {
                (_, Atom(_)) => Err("attempt to apply atom as formula"),
                (ref subject, Cell(ref b2)) => apply(subject, &b2.0, &b2.1),
            }
        },
    }
}

fn apply<'a>(subject: &'a Noun, op: &'a Noun, formula: &'a Noun) -> Result<&'a Noun, &'static str> {
  Err("almost")
}

    /*
    if let Cell(Box(subject, formula)) = n {
        match formula {
            Atom(_) => Err("attempt to apply atom as formula"),
            Cell(Cell(_,_), _) => Err("autocons is not yet implemented"),
            Cell(Atom(op), body) => run_op(subject, op, body),
        }
    } else {
        Err("attempt to evaluate atom")
    }
}

fn run_op(subject: &Noun, op: u128, body: &Noun) {
    match op {
        1 => Ok(body),
        _ => Err("unimplemented opcode"),
    }
}
*/


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
