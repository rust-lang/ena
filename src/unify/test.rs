extern crate test;
use self::test::Bencher;
use std::collections::HashSet;
use unify::{UnifyKey, UnificationTable};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct UnitKey(u32);

impl UnifyKey for UnitKey {
    type Value = ();
    fn index(&self) -> u32 {
        self.0
    }
    fn from_index(u: u32) -> UnitKey {
        UnitKey(u)
    }
    fn tag() -> &'static str {
        "UnitKey"
    }
}

#[test]
fn basic() {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let k1 = ut.new_key(());
    let k2 = ut.new_key(());
    assert_eq!(ut.unioned(k1, k2), false);
    ut.union(k1, k2);
    assert_eq!(ut.unioned(k1, k2), true);
}

#[test]
fn big_array() {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let mut keys = Vec::new();
    const MAX: usize = 1 << 15;

    for _ in 0..MAX {
        keys.push(ut.new_key(()));
    }

    for i in 1..MAX {
        let l = keys[i - 1];
        let r = keys[i];
        ut.union(l, r);
    }

    for i in 0..MAX {
        assert!(ut.unioned(keys[0], keys[i]));
    }
}

#[bench]
fn big_array_bench(b: &mut Bencher) {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let mut keys = Vec::new();
    const MAX: usize = 1 << 15;

    for _ in 0..MAX {
        keys.push(ut.new_key(()));
    }

    b.iter(|| {
        for i in 1..MAX {
            let l = keys[i - 1];
            let r = keys[i];
            ut.union(l, r);
        }

        for i in 0..MAX {
            assert!(ut.unioned(keys[0], keys[i]));
        }
    })
}

#[test]
fn even_odd() {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let mut keys = Vec::new();
    const MAX: usize = 1 << 10;

    for i in 0..MAX {
        let key = ut.new_key(());
        keys.push(key);

        if i >= 2 {
            ut.union(key, keys[i - 2]);
        }
    }

    for i in 1..MAX {
        assert!(!ut.unioned(keys[i - 1], keys[i]));
    }

    for i in 2..MAX {
        assert!(ut.unioned(keys[i - 2], keys[i]));
    }
}

#[test]
fn even_odd_iter() {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let mut keys = Vec::new();
    const MAX: usize = 1 << 10;

    for i in 0..MAX {
        let key = ut.new_key(());
        keys.push(key);

        if i >= 2 {
            ut.union(key, keys[i - 2]);
        }
    }

    let even_keys: HashSet<UnitKey> = ut.unioned_keys(keys[22]).collect();

    assert_eq!(even_keys.len(), MAX / 2);

    for key in even_keys {
        assert!((key.0 & 1) == 0);
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct IntKey(u32);

impl UnifyKey for IntKey {
    type Value = Option<i32>;
    fn index(&self) -> u32 {
        self.0
    }
    fn from_index(u: u32) -> IntKey {
        IntKey(u)
    }
    fn tag() -> &'static str {
        "IntKey"
    }
}

#[test]
fn unify_same_int_twice() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_value(k2, 22).is_ok());
    assert!(ut.unify_var_var(k1, k2).is_ok());
    assert_eq!(ut.probe(k1), Some(22));
}

#[test]
fn unify_vars_then_int_indirect() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    assert!(ut.unify_var_var(k1, k2).is_ok());
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert_eq!(ut.probe(k2), Some(22));
}

#[test]
fn unify_vars_different_ints_1() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    assert!(ut.unify_var_var(k1, k2).is_ok());
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_value(k2, 23).is_err());
}

#[test]
fn unify_vars_different_ints_2() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    assert!(ut.unify_var_var(k2, k1).is_ok());
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_value(k2, 23).is_err());
}

#[test]
fn unify_distinct_ints_then_vars() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_value(k2, 23).is_ok());
    assert!(ut.unify_var_var(k2, k1).is_err());
}

#[test]
fn unify_root_value_1() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    let k3 = ut.new_key(None);
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_var(k1, k2).is_ok());
    assert!(ut.unify_var_value(k3, 23).is_ok());
    assert!(ut.unify_var_var(k1, k3).is_err());
}

#[test]
fn unify_root_value_2() {
    let mut ut: UnificationTable<IntKey> = UnificationTable::new();
    let k1 = ut.new_key(None);
    let k2 = ut.new_key(None);
    let k3 = ut.new_key(None);
    assert!(ut.unify_var_value(k1, 22).is_ok());
    assert!(ut.unify_var_var(k2, k1).is_ok());
    assert!(ut.unify_var_value(k3, 23).is_ok());
    assert!(ut.unify_var_var(k1, k3).is_err());
}
