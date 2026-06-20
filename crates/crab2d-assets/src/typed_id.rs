use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::AssetId;

/// An [`AssetId`] branded with its asset kind at compile time.
///
/// The type parameter `K` is one of the marker types from [`crate::asset_markers`]
/// (e.g. [`crate::Sprite`], [`crate::Audio`]).  Two `TypedAssetId`s with different
/// `K` are distinct types, so the compiler prevents mixing them up.
///
/// All impls (Copy, Eq, Ord, Hash) delegate to the inner [`AssetId`] and deliberately
/// ignore `K`, which has no runtime representation.
pub struct TypedAssetId<K> {
    id: AssetId,
    _kind: PhantomData<K>,
}

impl<K> TypedAssetId<K> {
    pub(crate) fn new(id: AssetId) -> Self {
        Self {
            id,
            _kind: PhantomData,
        }
    }

    /// Returns the underlying untyped [`AssetId`].
    pub fn id(self) -> AssetId {
        self.id
    }
}

// Manual impls so K is not required to impl these traits.

impl<K> Copy for TypedAssetId<K> {}

impl<K> Clone for TypedAssetId<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> PartialEq for TypedAssetId<K> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<K> Eq for TypedAssetId<K> {}

impl<K> PartialOrd for TypedAssetId<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for TypedAssetId<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<K> Hash for TypedAssetId<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<K> fmt::Debug for TypedAssetId<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypedAssetId({})", self.id.raw())
    }
}
