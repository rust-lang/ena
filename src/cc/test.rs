use cc::{CongruenceClosure, Key, Token};
use self::TypeStruct::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeStruct {
    Func(Type),
    Struct(u32),
    Variable(Token),
}

type Type = Box<TypeStruct>;

impl Key for Type {
    fn to_token(&self) -> Option<Token> {
        match **self {
            TypeStruct::Func(_) | TypeStruct::Struct(_) => None,
            TypeStruct::Variable(t) => Some(t),
        }
    }

    fn shallow_eq(&self, key: &Type) -> bool {
        match (&**self, &**key) {
            (&Func(_), &Func(_)) => true,
            (&Struct(i), &Struct(j)) => i == j,
            _ => false,
        }
    }

    fn successors(&self) -> Vec<Self> {
        match **self {
            Func(ref t) => vec![t.clone()],
            Struct(_) => vec![],
            Variable(_) => vec![],
        }
    }
}

struct Types;

impl Types {
    pub fn struct0() -> Type {
        Box::new(Struct(0))
    }

    pub fn struct1() -> Type {
        Box::new(Struct(1))
    }

    pub fn struct2() -> Type {
        Box::new(Struct(2))
    }

    pub fn func(t: Type) -> Type {
        Box::new(Func(t))
    }

    pub fn func_struct0() -> Type {
        Box::new(Func(Types::struct0()))
    }

    pub fn func_struct1() -> Type {
        Box::new(Func(Types::struct1()))
    }

    pub fn func_struct2() -> Type {
        Box::new(Func(Types::struct2()))
    }
}

fn inference_var<'tcx>(cc: &mut CongruenceClosure<Type>) -> Type {
    let token = cc.new_token(move |token| Box::new(TypeStruct::Variable(token)));
    cc.key(token).clone()
}

#[test]
fn simple_as_it_gets() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();
    assert!(cc.merged(Types::struct0(), Types::struct0()));
    assert!(!cc.merged(Types::struct0(), Types::struct1()));
    assert!(cc.merged(Types::struct1(), Types::struct1()));
    assert!(cc.merged(Types::func_struct0(), Types::func_struct0()));
    assert!(!cc.merged(Types::func_struct0(), Types::func_struct1()));
    assert!(cc.merged(Types::func_struct1(), Types::func_struct1()));
}

#[test]
fn union_vars() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();
    cc.merge(Types::struct0(), Types::struct1());
    assert!(cc.merged(Types::struct0(), Types::struct1()));
}

#[test]
fn union_func_then_test_var() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();
    cc.merge(Types::struct0(), Types::struct1());
    assert!(cc.merged(Types::struct0(), Types::struct1()));
}

#[test]
fn union_direct() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.add(Types::func_struct0());
    cc.add(Types::func_struct1());
    cc.add(Types::struct0());
    cc.add(Types::struct1());

    cc.merge(Types::struct0(), Types::struct1());
    assert!(cc.merged(Types::func_struct0(), Types::func_struct1()));
}

macro_rules! indirect_test {
    ($test_name:ident: $a:expr, $b:expr; $c:expr, $d:expr) => {
        #[test]
        fn $test_name() {
            // Variant 1: call `add` explicitly
            //
            // This caused bugs because nodes were pre-existing.
            {
                let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

                cc.add(Types::func_struct0());
                cc.add(Types::func_struct2());
                cc.add(Types::struct0());
                cc.add(Types::struct1());
                cc.add(Types::struct2());

                cc.merge($a, $b);
                cc.merge($c, $d);
                assert!(cc.merged(Types::func_struct0(), Types::func_struct2()));
            }

            // Variant 2: never call `add` explicitly
            //
            // This is more how we expect library to be used in practice.
            {
                let mut cc2: CongruenceClosure<Type> = CongruenceClosure::new();
                cc2.merge($a, $b);
                cc2.merge($c, $d);
                assert!(cc2.merged(Types::func_struct0(), Types::func_struct2()));
            }
        }
    }
}

// The indirect tests test for the case where we merge V0 and V1, and
// we merged V1 and V2, and we want to use this to conclude that
// Func(V0) and Func(V2) are merged -- but there is no node created for
// Func(V1).
indirect_test! { indirect_test_1: Types::struct1(), Types::struct2(); Types::struct1(), Types::struct0() }
indirect_test! { indirect_test_2: Types::struct2(), Types::struct1(); Types::struct1(), Types::struct0() }
indirect_test! { indirect_test_3: Types::struct1(), Types::struct2(); Types::struct0(), Types::struct1() }
indirect_test! { indirect_test_4: Types::struct2(), Types::struct1(); Types::struct0(), Types::struct1() }

// Here we determine that `Func(V0) == Func(V1)` because `V0==V1`,
// but we never add nodes for `Func(_)`.
#[test]
fn merged_no_add() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::struct0(), Types::struct1());

    assert!(cc.merged(Types::func_struct0(), Types::func_struct1()));
}

// Here we determine that `Func(V0) == Func(V2)` because `V0==V1==V2`,
// but we never add nodes for `Func(_)`.
#[test]
fn merged_no_add_indirect() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::struct0(), Types::struct1());
    cc.merge(Types::struct1(), Types::struct2());

    assert!(cc.merged(Types::func_struct0(), Types::func_struct2()));
}

// Here we determine that `Func(V0) == Func(V2)` because `V0==V1==V2`,
// but we never add nodes for `Func(_)`.
#[test]
fn func_not_merged() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::func_struct0(), Types::func_struct1());

    assert!(!cc.merged(Types::struct0(), Types::struct1()));
    assert!(cc.merged(Types::func_struct0(), Types::func_struct1()));
}

// Here we show that merging `Func(V1) == Func(V2)` does NOT imply that
// `V1 == V2`.
#[test]
fn merge_fns_not_inputs() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::func_struct0(), Types::func_struct1());

    assert!(!cc.merged(Types::struct0(), Types::struct1()));
    assert!(cc.merged(Types::func_struct0(), Types::func_struct1()));
}

#[test]
fn inf_var_union() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    let v0 = inference_var(&mut cc);
    let v1 = inference_var(&mut cc);
    let v2 = inference_var(&mut cc);
    let func_v0 = Types::func(v0.clone());
    let func_v1 = Types::func(v1.clone());
    let func_v2 = Types::func(v2.clone());

    cc.merge(v0.clone(), v1.clone());

    assert!(cc.map.is_empty()); // inf variables don't take up map slots

    assert!(cc.merged(func_v0.clone(), func_v1.clone()));
    assert!(!cc.merged(func_v0.clone(), func_v2.clone()));

    cc.merge(func_v0.clone(), func_v2.clone());
    assert!(cc.merged(func_v0.clone(), func_v2.clone()));
    assert!(cc.merged(func_v1.clone(), func_v2.clone()));

    assert_eq!(cc.map.len(), 3); // each func needs an entry
}

#[test]
fn struct_union_no_add() {

    // This particular pattern of unifications exploits a potentially
    // subtle bug:
    // - We merge `Types::struct0()` and `Types::struct1()`
    //   and then merge `FUNC(Types::struct0())` and `FUNC(Types::struct2())`.
    // - From this we should be able to deduce that `FUNC(Types::struct1()) == FUNC(Types::struct2())`.
    // - However, if we are not careful with accounting for
    //   predecessors and so forth, this fails. For example, when
    //   adding `FUNC(Types::struct1())`, we have to consider `FUNC(Types::struct0())`
    //   to be a predecessor of `Types::struct1()`.

    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::struct0(), Types::struct1());
    assert!(cc.merged(Types::func_struct0(), Types::func_struct1()));
    assert!(!cc.merged(Types::func_struct0(), Types::func_struct2()));

    cc.merge(Types::func_struct0(), Types::func_struct2());
    assert!(cc.merged(Types::func_struct0(), Types::func_struct2()));
    assert!(cc.merged(Types::func_struct1(), Types::func_struct2()));
}

#[test]
fn merged_keys() {
    let mut cc: CongruenceClosure<Type> = CongruenceClosure::new();

    cc.merge(Types::struct0(), Types::struct1());
    cc.merge(Types::func_struct0(), Types::func_struct2());

    // Here we don't yet see `Types::func_struct1()` because it has no
    // corresponding node:
    let keys: Vec<Type> = cc.merged_keys(Types::func_struct2()).collect();
    assert_eq!(&keys[..], &[Types::func_struct2(), Types::func_struct0()]);

    // But of course `merged` returns true (and adds a node):
    assert!(cc.merged(Types::func_struct1(), Types::func_struct2()));

    // So now we see it:
    let keys: Vec<Type> = cc.merged_keys(Types::func_struct2()).collect();
    assert_eq!(&keys[..], &[Types::func_struct2(), Types::func_struct1(), Types::func_struct0()]);
}

