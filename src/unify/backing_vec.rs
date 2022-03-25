#[cfg(feature = "persistent")]
use dogged::DVec;
use snapshot_vec as sv;
use std::marker::PhantomData;
use std::ops::{self, Range};

use undo_log::{Rollback, Snapshots, UndoLogs, VecLog};

use super::{ExtraTraversalData, NoExtraTraversalData, UnifyKey, UnifyValue, VarValue};

#[allow(dead_code)] // rustc BUG
#[allow(type_alias_bounds)]
type Key<S: UnificationStoreBase> = <S as UnificationStoreBase>::Key;

/// Largely internal trait implemented by the unification table
/// backing store types. The most common such type is `InPlace`,
/// which indicates a standard, mutable unification table.
pub trait UnificationStoreBase:
    ops::Index<usize, Output = VarValue<Key<Self>, Self::ExtraTraversalData>>
{
    type Key: UnifyKey<Value = Self::Value>;
    type Value: UnifyValue;
    type ExtraTraversalData: ExtraTraversalData<Self::Key>;

    fn len(&self) -> usize;

    fn tag() -> &'static str {
        Self::Key::tag()
    }
}

pub trait UnificationStoreMut: UnificationStoreBase {
    fn reset_unifications(
        &mut self,
        value: impl FnMut(u32) -> VarValue<Self::Key, Self::ExtraTraversalData>,
    );

    fn push(&mut self, value: VarValue<Self::Key, Self::ExtraTraversalData>);

    fn reserve(&mut self, num_new_values: usize);

    fn update<F>(&mut self, index: usize, op: F)
    where
        F: FnOnce(&mut VarValue<Self::Key, Self::ExtraTraversalData>);
}

pub trait UnificationStore: UnificationStoreMut {
    type Snapshot;

    fn start_snapshot(&mut self) -> Self::Snapshot;

    fn rollback_to(&mut self, snapshot: Self::Snapshot);

    fn commit(&mut self, snapshot: Self::Snapshot);

    fn values_since_snapshot(&self, snapshot: &Self::Snapshot) -> Range<usize>;
}

/// Backing store for an in-place unification table.
/// Not typically used directly.
#[derive(Clone, Debug)]
pub struct InPlace<
    K: UnifyKey,
    TD: ExtraTraversalData<K> = NoExtraTraversalData,
    V: sv::VecLike<Delegate<K, TD>> = Vec<VarValue<K, TD>>,
    L = VecLog<sv::UndoLog<Delegate<K, TD>>>,
> {
    pub(crate) values: sv::SnapshotVec<Delegate<K, TD>, V, L>,
}

// HACK(eddyb) manual impl avoids `Default` bound on `K`.
impl<
        K: UnifyKey,
        TD: ExtraTraversalData<K>,
        V: sv::VecLike<Delegate<K, TD>> + Default,
        L: Default,
    > Default for InPlace<K, TD, V, L>
{
    fn default() -> Self {
        InPlace {
            values: sv::SnapshotVec::new(),
        }
    }
}

impl<K, TD, V, L> UnificationStoreBase for InPlace<K, TD, V, L>
where
    K: UnifyKey,
    V: sv::VecLike<Delegate<K, TD>>,
    TD: ExtraTraversalData<K>,
{
    type Key = K;
    type Value = K::Value;
    type ExtraTraversalData = TD;

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<K, TD, V, L> UnificationStoreMut for InPlace<K, TD, V, L>
where
    K: UnifyKey,
    TD: ExtraTraversalData<K>,
    V: sv::VecLike<Delegate<K, TD>>,
    L: UndoLogs<sv::UndoLog<Delegate<K, TD>>>,
{
    #[inline]
    fn reset_unifications(
        &mut self,
        mut value: impl FnMut(u32) -> VarValue<Self::Key, Self::ExtraTraversalData>,
    ) {
        self.values.set_all(|i| value(i as u32));
    }

    #[inline]
    fn push(&mut self, value: VarValue<Self::Key, Self::ExtraTraversalData>) {
        self.values.push(value);
    }

    #[inline]
    fn reserve(&mut self, num_new_values: usize) {
        self.values.reserve(num_new_values);
    }

    #[inline]
    fn update<F>(&mut self, index: usize, op: F)
    where
        F: FnOnce(&mut VarValue<Self::Key, Self::ExtraTraversalData>),
    {
        self.values.update(index, op)
    }
}

impl<K, TD, V, L> UnificationStore for InPlace<K, TD, V, L>
where
    K: UnifyKey,
    TD: ExtraTraversalData<K>,
    V: sv::VecLike<Delegate<K, TD>>,
    L: Snapshots<sv::UndoLog<Delegate<K, TD>>>,
{
    type Snapshot = sv::Snapshot<L::Snapshot>;

    #[inline]
    fn start_snapshot(&mut self) -> Self::Snapshot {
        self.values.start_snapshot()
    }

    #[inline]
    fn rollback_to(&mut self, snapshot: Self::Snapshot) {
        self.values.rollback_to(snapshot);
    }

    #[inline]
    fn commit(&mut self, snapshot: Self::Snapshot) {
        self.values.commit(snapshot);
    }

    #[inline]
    fn values_since_snapshot(&self, snapshot: &Self::Snapshot) -> Range<usize> {
        snapshot.value_count..self.len()
    }
}

impl<K, TD, V, L> ops::Index<usize> for InPlace<K, TD, V, L>
where
    V: sv::VecLike<Delegate<K, TD>>,
    K: UnifyKey,
    TD: ExtraTraversalData<K>,
{
    type Output = VarValue<K, TD>;
    fn index(&self, index: usize) -> &VarValue<K, TD> {
        &self.values[index]
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct Delegate<K, TD>(PhantomData<(K, TD)>);

impl<K: UnifyKey, TD: ExtraTraversalData<K>> sv::SnapshotVecDelegate for Delegate<K, TD> {
    type Value = VarValue<K, TD>;
    type Undo = ();

    fn reverse(_: &mut Vec<VarValue<K, TD>>, _: ()) {}
}

impl<K: UnifyKey, TD: ExtraTraversalData<K>> Rollback<sv::UndoLog<Delegate<K, TD>>>
    for super::UnificationTableStorage<K, TD>
{
    fn reverse(&mut self, undo: sv::UndoLog<Delegate<K, TD>>) {
        self.values.values.reverse(undo);
    }
}

#[cfg(feature = "persistent")]
#[derive(Clone, Debug)]
pub struct Persistent<K: UnifyKey, TD: ExtraTraversalData<K>> {
    values: DVec<VarValue<K, TD>>,
}

// HACK(eddyb) manual impl avoids `Default` bound on `K`.
#[cfg(feature = "persistent")]
impl<K: UnifyKey, TD: ExtraTraversalData<K>> Default for Persistent<K, TD> {
    fn default() -> Self {
        Persistent {
            values: DVec::new(),
        }
    }
}

#[cfg(feature = "persistent")]
impl<K: UnifyKey, TD: ExtraTraversalData<K>> UnificationStoreBase for Persistent<K, TD> {
    type Key = K;
    type Value = K::Value;
    type ExtraTraversalData = TD;

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(feature = "persistent")]
impl<K: UnifyKey, TD: ExtraTraversalData<K>> UnificationStoreMut for Persistent<K, TD> {
    #[inline]
    fn reset_unifications(
        &mut self,
        mut value: impl FnMut(u32) -> VarValue<Self::Key, Self::ExtraTraversalData>,
    ) {
        // Without extending dogged, there isn't obviously a more
        // efficient way to do this. But it's pretty dumb. Maybe
        // dogged needs a `map`.
        for i in 0..self.values.len() {
            self.values[i] = value(i as u32);
        }
    }

    #[inline]
    fn push(&mut self, value: VarValue<Self::Key, Self::ExtraTraversalData>) {
        self.values.push(value);
    }

    #[inline]
    fn reserve(&mut self, _num_new_values: usize) {
        // not obviously relevant to DVec.
    }

    #[inline]
    fn update<F>(&mut self, index: usize, op: F)
    where
        F: FnOnce(&mut VarValue<Self::Key, Self::ExtraTraversalData>),
    {
        let p = &mut self.values[index];
        op(p);
    }
}

#[cfg(feature = "persistent")]
impl<K: UnifyKey, TD: ExtraTraversalData<K>> UnificationStore for Persistent<K, TD> {
    type Snapshot = Self;

    #[inline]
    fn start_snapshot(&mut self) -> Self::Snapshot {
        self.clone()
    }

    #[inline]
    fn rollback_to(&mut self, snapshot: Self::Snapshot) {
        *self = snapshot;
    }

    #[inline]
    fn commit(&mut self, _snapshot: Self::Snapshot) {}

    #[inline]
    fn values_since_snapshot(&self, snapshot: &Self::Snapshot) -> Range<usize> {
        snapshot.len()..self.len()
    }
}

#[cfg(feature = "persistent")]
impl<K, TD> ops::Index<usize> for Persistent<K, TD>
where
    K: UnifyKey,
    TD: ExtraTraversalData<K>,
{
    type Output = VarValue<K, TD>;
    fn index(&self, index: usize) -> &VarValue<K, TD> {
        &self.values[index]
    }
}
