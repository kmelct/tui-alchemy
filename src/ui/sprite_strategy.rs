#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpriteSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteRole {
    AtlasTile { tile_width: u16, tile_height: u16 },
    IngredientSlot,
    ResultSlot,
    DragGhost,
}

impl SpriteRole {
    pub(super) fn sprite_size(self) -> SpriteSize {
        match self {
            Self::AtlasTile {
                tile_width,
                tile_height,
            } => SpriteSize {
                width: if tile_width > 10 {
                    u32::from(tile_width.saturating_sub(2).clamp(12, 18))
                } else {
                    8
                },
                height: if tile_height > 6 { 12 } else { 10 },
            },
            Self::IngredientSlot | Self::DragGhost => SpriteSize {
                width: 8,
                height: 8,
            },
            Self::ResultSlot => SpriteSize {
                width: 12,
                height: 12,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SpriteRole;

    #[test]
    fn atlas_role_scales_sprite_to_tile_body() {
        let size = SpriteRole::AtlasTile {
            tile_width: 14,
            tile_height: 10,
        }
        .sprite_size();

        assert_eq!(size.width, 12);
        assert_eq!(size.height, 12);
    }

    #[test]
    fn workbench_roles_use_distinct_sprite_sizes() {
        assert_eq!(SpriteRole::IngredientSlot.sprite_size().width, 8);
        assert_eq!(SpriteRole::ResultSlot.sprite_size().width, 12);
        assert_eq!(SpriteRole::DragGhost.sprite_size().height, 8);
    }
}
