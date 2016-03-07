use cc::{CongruenceClosure, Key, Token};
use self::TypeStruct::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeStruct<'tcx> {
    Func(Type<'tcx>),
    Struct(u32),
    Variable(Token),
}

type Type<'tcx> = &'tcx TypeStruct<'tcx>;

impl<'tcx> Key for Type<'tcx> {
    fn to_token(&self) -> Option<Token> {
        match *self {
            &TypeStruct::Func(_) | &TypeStruct::Struct(_) => None,
            &TypeStruct::Variable(t) => Some(t),
        }
    }

    fn shallow_eq(&self, key: &Type<'tcx>) -> bool {
        match (*self, *key) {
            (&Func(_), &Func(_)) => true,
            (&Struct(i), &Struct(j)) => i == j,
            _ => false,
        }
    }

    fn successors(&self) -> Vec<Self> {
        match *self {
            &Func(t) => vec![t],
            &Struct(_) => vec![],
            &Variable(_) => vec![],
        }
    }
}

const STRUCT_0: Type<'static> = &Struct(0);
const FUNC_STRUCT_0: Type<'static> = &Func(STRUCT_0);
const STRUCT_1: Type<'static> = &Struct(1);
const FUNC_STRUCT_1: Type<'static> = &Func(STRUCT_1);
const STRUCT_2: Type<'static> = &Struct(2);
const FUNC_STRUCT_2: Type<'static> = &Func(STRUCT_2);

fn inference_var<'tcx>(cc: &mut CongruenceClosure<Type<'tcx>>,
                       storage: &'tcx mut Option<TypeStruct<'tcx>>)
                       -> Type<'tcx> {
    let token = cc.new_token(move |token| {
        *storage = Some(TypeStruct::Variable(token));
        storage.as_ref().unwrap()
    });
    *cc.key(token)
}

#[test]
fn simple_as_it_gets() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    assert!(cc.merged(STRUCT_0, STRUCT_0));
    assert!(!cc.merged(STRUCT_0, STRUCT_1));
    assert!(cc.merged(STRUCT_1, STRUCT_1));
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_0));
    assert!(!cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
    assert!(cc.merged(FUNC_STRUCT_1, FUNC_STRUCT_1));
}

#[test]
fn union_vars() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    cc.merge(STRUCT_0, STRUCT_1);
    assert!(cc.merged(STRUCT_0, STRUCT_1));
}

#[test]
fn union_func_then_test_var() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
    cc.merge(STRUCT_0, STRUCT_1);
    assert!(cc.merged(STRUCT_0, STRUCT_1));
}

#[test]
fn union_direct() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.add(FUNC_STRUCT_0);
    cc.add(FUNC_STRUCT_1);
    cc.add(STRUCT_0);
    cc.add(STRUCT_1);

    cc.merge(STRUCT_0, STRUCT_1);
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
}

macro_rules! indirect_test {
    ($test_name:ident: $a:expr, $b:expr; $c:expr, $d:expr) => {
        #[test]
        fn $test_name() {
            // Variant 1: call `add` explicitly
            //
            // This caused bugs because nodes were pre-existing.
            {
                let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

                cc.add(FUNC_STRUCT_0);
                cc.add(FUNC_STRUCT_2);
                cc.add(STRUCT_0);
                cc.add(STRUCT_1);
                cc.add(STRUCT_2);

                cc.merge($a, $b);
                cc.merge($c, $d);
                assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_2));
            }

            // Variant 2: never call `add` explicitly
            //
            // This is more how we expect library to be used in practice.
            {
                let mut cc2: CongruenceClosure<Type<'static>> = CongruenceClosure::new();
                cc2.merge($a, $b);
                cc2.merge($c, $d);
                assert!(cc2.merged(FUNC_STRUCT_0, FUNC_STRUCT_2));
            }
        }
    }
}

// The indirect tests test for the case where we merge V0 and V1, and
// we merged V1 and V2, and we want to use this to conclude that
// Func(V0) and Func(V2) are merged -- but there is no node created for
// Func(V1).
indirect_test! { indirect_test_1: STRUCT_1, STRUCT_2; STRUCT_1, STRUCT_0 }
indirect_test! { indirect_test_2: STRUCT_2, STRUCT_1; STRUCT_1, STRUCT_0 }
indirect_test! { indirect_test_3: STRUCT_1, STRUCT_2; STRUCT_0, STRUCT_1 }
indirect_test! { indirect_test_4: STRUCT_2, STRUCT_1; STRUCT_0, STRUCT_1 }

// Here we determine that `Func(V0) == Func(V1)` because `V0==V1`,
// but we never add nodes for `Func(_)`.
#[test]
fn merged_no_add() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.merge(STRUCT_0, STRUCT_1);

    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
}

// Here we determine that `Func(V0) == Func(V2)` because `V0==V1==V2`,
// but we never add nodes for `Func(_)`.
#[test]
fn merged_no_add_indirect() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.merge(STRUCT_0, STRUCT_1);
    cc.merge(STRUCT_1, STRUCT_2);

    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_2));
}

// Here we determine that `Func(V0) == Func(V2)` because `V0==V1==V2`,
// but we never add nodes for `Func(_)`.
#[test]
fn func_not_merged() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.merge(FUNC_STRUCT_0, FUNC_STRUCT_1);

    assert!(!cc.merged(STRUCT_0, STRUCT_1));
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
}

// Here we show that merging `Func(V1) == Func(V2)` does NOT imply that
// `V1 == V2`.
#[test]
fn merge_fns_not_inputs() {
    let mut cc: CongruenceClosure<Type<'static>> = CongruenceClosure::new();

    cc.merge(FUNC_STRUCT_0, FUNC_STRUCT_1);

    assert!(!cc.merged(STRUCT_0, STRUCT_1));
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
}

#[test]
fn inf_var_union() {
    // These variables serve as a kind of arena. They have be declared
    // in a tuple like this for the temporary lifetimes to work out.
    let (mut v0, mut v1, mut v2, func_v0, func_v1, func_v2);

    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    v0 = None;
    v1 = None;
    v2 = None;
    let v0 = inference_var(&mut cc, &mut v0);
    let v1 = inference_var(&mut cc, &mut v1);
    let v2 = inference_var(&mut cc, &mut v2);
    func_v0 = TypeStruct::Func(v0);
    func_v1 = TypeStruct::Func(v1);
    func_v2 = TypeStruct::Func(v2);

    cc.merge(v0, v1);

    assert!(cc.map.is_empty()); // inf variables don't take up map slots

    assert!(cc.merged(&func_v0, &func_v1));
    assert!(!cc.merged(&func_v0, &func_v2));

    cc.merge(&func_v0, &func_v2);
    assert!(cc.merged(&func_v0, &func_v2));
    assert!(cc.merged(&func_v1, &func_v2));

    assert_eq!(cc.map.len(), 3); // each func needs an entry
}


#[test]
fn struct_union_no_add() {

    // This particular pattern of unifications exploits a potentially
    // subtle bug:
    // - We merge `STRUCT_0` and `STRUCT_1`
    //   and then merge `FUNC(STRUCT_0)` and `FUNC(STRUCT_2)`.
    // - From this we should be able to deduce that `FUNC(STRUCT_1) == FUNC(STRUCT_2)`.
    // - However, if we are not careful with accounting for
    //   predecessors and so forth, this fails. For example, when
    //   adding `FUNC(STRUCT_1)`, we have to consider `FUNC(STRUCT_0)`
    //   to be a predecessor of `STRUCT_1`.

    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(STRUCT_0, STRUCT_1);
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_1));
    assert!(!cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_2));

    cc.merge(FUNC_STRUCT_0, FUNC_STRUCT_2);
    assert!(cc.merged(FUNC_STRUCT_0, FUNC_STRUCT_2));
    assert!(cc.merged(FUNC_STRUCT_1, FUNC_STRUCT_2));
}

