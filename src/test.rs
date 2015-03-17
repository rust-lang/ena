use unify::{UnifyKey, UnifyValue, UnificationTable};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct UnitKey(u32);

impl UnifyKey for UnitKey {
    type Value = ();
    fn index(&self) -> u32 { self.0 }
    fn from_index(u: u32) -> UnitKey { UnitKey(u) }
    fn tag(_: Option<UnitKey>) -> &'static str { "UnitKey" }
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
    const max: usize = 1 << 20;

    for _ in 0..max {
        keys.push(ut.new_key(()));
    }

    for i in 1..max {
        let l = keys[i-1];
        let r = keys[i];
        ut.union(l, r);
    }

    for i in 0..max {
        assert!(ut.unioned(keys[0], keys[i]));
    }
}

#[test]
fn even_odd() {
    let mut ut: UnificationTable<UnitKey> = UnificationTable::new();
    let mut keys = Vec::new();
    const max: usize = 1 << 10;

    for i in 0..max {
        let key = ut.new_key(());
        keys.push(key);

        if i >= 2 {
            ut.union(key, keys[i-2]);
        }
    }

    for i in 1..max {
        assert!(!ut.unioned(keys[i-1], keys[i]));
    }

    for i in 2..max {
        assert!(ut.unioned(keys[i-2], keys[i]));
    }
}
