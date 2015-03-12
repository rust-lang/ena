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
    assert_eq!(ut.unioned(&k1, &k2), false);
    ut.union(&k1, &k2);
    assert_eq!(ut.unioned(&k1, &k2), true);
}

