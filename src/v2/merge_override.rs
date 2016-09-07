//! Merging two `docker-compose.yml` files together.

use std::collections::BTreeMap;
use std::collections::btree_map;
use std::marker::{PhantomData, Sized};

/// This trait is implemented by types that can be merged
pub trait MergeOverride: Clone + Sized {
    /// Given this value and an override value, merge the override value
    /// into this one, producing a new value.
    ///
    /// The default implementation just ignores the original value in favor
    /// of `ovr`, so it's appropriate for simple scalar types.  More
    /// complex types will require a custom implementation.
    fn merge_override(&self, ovr: &Self) -> Self {
        ovr.to_owned()
    }
}

macro_rules! assert_merge {
    ($ty:ty, $value1:expr, $value2:expr, $expected: expr) => {
        assert_eq!(($value1 as $ty).merge_override(&($value2 as $ty)),
                   ($expected as $ty));
    }
}

impl MergeOverride for u16 {}
impl MergeOverride for u32 {}
impl MergeOverride for bool {}
impl MergeOverride for String {}
impl<T> MergeOverride for PhantomData<T> {}

#[test]
fn simple_types_are_replaced() {
    assert_merge!(u16, 1, 2, 2);
    assert_merge!(bool, false, true, true);
    assert_merge!(String, "s1".to_owned(), "s2".to_owned(), "s2".to_owned());
}

impl<T: MergeOverride> MergeOverride for Option<T> {
    /// If both option values are present, call `merge_override`
    /// recursively.  If only one value is present, use that one.  If
    /// neither is present, return `None`.
    fn merge_override(&self, ovr: &Self) -> Self {
        match (self, ovr) {
            (&Some(ref x), &Some(ref y)) => Some(x.merge_override(y)),
            (&Some(ref x), &None)        => Some(x.to_owned()),
            (&None,        &Some(ref y)) => Some(y.to_owned()),
            (&None,        &None)        => None,
        }
    }
}

#[test]
fn option_uses_second_value_if_not_none() {
    assert_merge!(Option<bool>, Some(false), Some(true), Some(true));
    assert_merge!(Option<bool>, Some(false), None,       Some(false));
    assert_merge!(Option<bool>, None,        Some(true), Some(true));
    assert_merge!(Option<bool>, None,        None,       None);

    // Check complex recursive merge, too.
    assert_merge!(Option<Vec<bool>>,
                  Some(vec!(false)), Some(vec!(true)),
                  Some(vec!(false, true)));
}

impl<T: MergeOverride> MergeOverride for Vec<T> {
    /// Appends `ovr` to `self`, creating a new list.
    fn merge_override(&self, ovr: &Self) -> Self {
        let mut result = self.clone();
        result.extend_from_slice(ovr as &[T]);
        result
    }
}

#[test]
fn vec_appends_new_values() {
    assert_merge!(Vec<bool>, vec!(false), vec!(true), vec!(false, true));
}

impl<K: Ord+Clone, T: MergeOverride> MergeOverride for BTreeMap<K, T> {
    /// Merges `ovr` into `self`, recursively merging values if present in
    /// both, and returns a new `BTreeMap`.
    fn merge_override(&self, ovr: &Self) -> Self {
        let mut result = self.clone();
        for (ovr_key, ovr_val) in ovr {
            match result.entry(ovr_key.to_owned()) {
                btree_map::Entry::Vacant(vacant) => {
                    vacant.insert(ovr_val.clone());
                }
                btree_map::Entry::Occupied(mut occupied) => {
                    let merged = occupied.get().merge_override(ovr_val);
                    occupied.insert(merged);
                }
            }
        }
        result
    }
}

#[test]
fn btree_map_merges_by_key() {
    let mut map1 = BTreeMap::new();
    map1.insert("a", vec!(false));
    map1.insert("c", vec!(false));
    let mut map2 = BTreeMap::new();
    map2.insert("b", vec!(true));
    map2.insert("c", vec!(true));
    let mut expected = BTreeMap::new();
    expected.insert("a", vec!(false));
    expected.insert("b", vec!(true));
    expected.insert("c", vec!(false, true));
    assert_merge!(BTreeMap<&'static str, Vec<bool>>, map1, map2, expected);
}

/// Derive `MergeOverride` for a custom struct type, by recursively merging
/// all field.
macro_rules! derive_merge_override_for {
    ($ty:ident, { $( $field:ident ),+ }) => {
        /// Recursive merge all fields in the structure.
        impl MergeOverride for $ty {
            fn merge_override(&self, ovr: &Self) -> Self {
                $ty {
                    $( $field: self.$field.merge_override(&ovr.$field) ),+
                }
            }
        }
    }
}
