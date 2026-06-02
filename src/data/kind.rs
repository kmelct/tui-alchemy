#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogKind {
    LittleAlchemy1,
    LittleAlchemy2,
    Combined,
}

impl CatalogKind {
    pub const fn title(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 => "Little Alchemy 1",
            Self::LittleAlchemy2 => "Little Alchemy 2",
            Self::Combined => "Little Alchemy",
        }
    }

    pub const fn asset_dir(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "assets/icons/little-alchemy-1",
            Self::LittleAlchemy2 => "assets/icons/little-alchemy-2",
        }
    }

    pub const fn asset_extension(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "png",
            Self::LittleAlchemy2 => "svg",
        }
    }

    pub const fn pixel_sprite_dir(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "assets/pixel-sprites/little-alchemy-1",
            Self::LittleAlchemy2 => "assets/pixel-sprites/little-alchemy-2",
        }
    }
}
