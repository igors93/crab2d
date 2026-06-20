use crate::AssetKind;

// Prevents external crates from implementing AssetType.
mod sealed {
    pub trait Sealed {}
}

/// Compile-time marker that associates a zero-sized type with a runtime [`AssetKind`].
///
/// Sealed — only the built-in markers in this module implement this trait.
/// To add a new kind, extend [`AssetKind`] and add a new marker below.
pub trait AssetType: sealed::Sealed + 'static {
    fn kind() -> AssetKind;
}

macro_rules! define_asset_markers {
    ($($marker:ident => $variant:ident),* $(,)?) => {
        $(
            pub struct $marker;

            impl sealed::Sealed for $marker {}

            impl AssetType for $marker {
                #[inline]
                fn kind() -> AssetKind {
                    AssetKind::$variant
                }
            }
        )*
    };
}

define_asset_markers! {
    Sprite  => Sprite,
    Tilemap => Tilemap,
    Audio   => Audio,
    Script  => Script,
    Config  => Config,
}
