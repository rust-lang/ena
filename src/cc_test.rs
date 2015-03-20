use cc::{CongruenceClosure, Key};
use self::TypeStruct::*;

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
enum TypeStruct<'tcx> {
    Box(Type<'tcx>),
    Ref(Type<'tcx>),
    Pair(Type<'tcx>, Type<'tcx>),
    Int,
    Variable(u32),
}

type Type<'tcx> = &'tcx TypeStruct<'tcx>;

impl<'tcx> Key for Type<'tcx> {
    fn shallow_eq(&self, key: &Type<'tcx>) -> bool {
        match (*self, *key) {
            (&Box(_), &Box(_)) => true,
            (&Ref(_), &Ref(_)) => true,
            (&Pair(..), &Pair(..)) => true,
            (&Int, &Int) => true,
            (&Variable(i), &Variable(j)) => i == j,
            _ => false,
        }
    }

    fn successors(&self) -> Vec<Self> {
        match *self {
            &Box(t) => vec![t],
            &Ref(t) => vec![t],
            &Pair(t, u) => vec![t, u],
            &Int => vec![],
            &Variable(_) => vec![],
        }
    }
}

const INT: Type<'static> = &Int;
const BOX_INT: Type<'static> = &Box(INT);
const VAR_0: Type<'static> = &Variable(0);
const BOX_VAR_0: Type<'static> = &Box(VAR_0);
const VAR_1: Type<'static> = &Variable(1);
const BOX_VAR_1: Type<'static> = &Box(VAR_1);
const VAR_2: Type<'static> = &Variable(2);
const BOX_VAR_2: Type<'static> = &Box(VAR_2);

#[test]
fn simple_as_it_gets() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    assert!(cc.merged(INT, INT));
    assert!(cc.merged(VAR_0, VAR_0));
    assert!(cc.merged(BOX_VAR_1, BOX_VAR_1));
}

#[test]
fn union_vars() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    cc.merge(VAR_0, VAR_1);
    assert!(cc.merged(VAR_0, VAR_1));
}

#[test]
fn union_box_then_test_var() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    cc.merge(VAR_0, VAR_1);
    assert!(cc.merged(VAR_0, VAR_1));
}

#[test]
fn union_direct() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.add(BOX_VAR_0);
    cc.add(BOX_VAR_1);
    cc.add(VAR_0);
    cc.add(VAR_1);

    cc.merge(VAR_0, VAR_1);
    assert!(cc.merged(BOX_VAR_0, BOX_VAR_1));
}

macro_rules! indirect_test {
    ($a:expr, $b:expr; $c:expr, $d:expr) => {
        #[test]
        fn union_indirect_1() {
            let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

            cc.add(BOX_VAR_0);
            cc.add(BOX_VAR_1);
            cc.add(BOX_VAR_2);
            cc.add(VAR_0);
            cc.add(VAR_1);
            cc.add(VAR_2);

            cc.merge($a, $b);
            cc.merge($c, $d);
            assert!(cc.merged(BOX_VAR_0, BOX_VAR_2));
        }
    }
}

indirect_test! { VAR_2, VAR_1; VAR_1, VAR_0 }

