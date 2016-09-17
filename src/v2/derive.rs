//! Custom macro rules that act sort of like `derive` clauses.

macro_rules! derive_standard_impls_for {
    ($ty:ident, { $( $field:ident ),+ }) => {
        derive_interpolate_all_for!($ty, { $( $field ),+ });
        derive_merge_override_for!($ty, { $( $field ),+ });
    }
}
