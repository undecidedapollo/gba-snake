use core::ptr::copy_nonoverlapping;

use gba::{
    Align4, include_aligned_bytes,
    mmio::{BG_PALETTE, CHARBLOCK0_8BPP, OBJ_ATTR_ALL, OBJ_PALETTE, TEXT_SCREENBLOCKS},
    prelude::{ObjAttr, ObjDisplayStyle},
    video::Color,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub static SHARED_PALETTE: Align4<[u8; 20]> =
    include_aligned_bytes!("../../asset_out/shared.palette");

pub static SNAKE_HEAD_UP: Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_up.sprite");
pub static SNAKE_HEAD_LEFT: Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_left.sprite");
pub static SNAKE_HEAD_RIGHT: Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_right.sprite");
pub static SNAKE_HEAD_DOWN: Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_down.sprite");
pub static SNAKE_BODY_1: Align4<[u8; 64]> = include_aligned_bytes!("../../asset_out/body_1.sprite");

pub static FRUIT_CHERRY: Align4<[u8; 64]> = include_aligned_bytes!("../../asset_out/cherry.sprite");
pub static FRUIT_BANANA: Align4<[u8; 64]> = include_aligned_bytes!("../../asset_out/banana.sprite");

#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum AssetObjTile {
    SnakeHeadUp = 0,
    SnakeHeadLeft = 2,
    SnakeHeadRight = 4,
    SnakeHeadDown = 6,
    SnakeBody1 = 8,
}

impl Into<usize> for AssetObjTile {
    fn into(self) -> usize {
        return self as usize;
    }
}
impl Into<u16> for AssetObjTile {
    fn into(self) -> u16 {
        return self as u16;
    }
}

#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum AssetBgTile {
    Blank = 0,
    Cherry = 1,
    Banana = 2,
}

impl Into<usize> for AssetBgTile {
    fn into(self) -> usize {
        return self as usize;
    }
}
impl Into<u16> for AssetBgTile {
    fn into(self) -> u16 {
        return self as u16;
    }
}

pub fn reset_data() {
    let mut ottr = ObjAttr::new();
    ottr.0 = ottr.0.with_style(ObjDisplayStyle::NotDisplayed);
    OBJ_ATTR_ALL.iter().for_each(|va| va.write(ottr));

    let zeros: [u32; 32] = core::array::repeat(0);
    // Zero out the background
    for i in 0..16 {
        unsafe {
            copy_nonoverlapping(
                zeros.as_ptr(),
                TEXT_SCREENBLOCKS
                    .get_frame(1)
                    .unwrap()
                    .get_row(i * 2)
                    .unwrap()
                    .as_usize() as *mut u32,
                zeros.len(),
            );
            copy_nonoverlapping(
                zeros.as_ptr(),
                TEXT_SCREENBLOCKS
                    .get_frame(2)
                    .unwrap()
                    .get_row(i * 2)
                    .unwrap()
                    .as_usize() as *mut u32,
                zeros.len(),
            );
            copy_nonoverlapping(
                zeros.as_ptr(),
                TEXT_SCREENBLOCKS
                    .get_frame(3)
                    .unwrap()
                    .get_row(i * 2)
                    .unwrap()
                    .as_usize() as *mut u32,
                zeros.len(),
            );
        }
    }

    // Make the zero-th tile transparent
    unsafe {
        copy_nonoverlapping(
            zeros.as_ptr(),
            CHARBLOCK0_8BPP.index(AssetBgTile::Blank.into()).as_usize() as *mut u32,
            16,
        );
        copy_nonoverlapping(
            SHARED_PALETTE.0.as_ptr(),
            OBJ_PALETTE.as_usize() as *mut u8,
            SHARED_PALETTE.0.len(),
        );
        copy_nonoverlapping(
            SHARED_PALETTE.0.as_ptr(),
            BG_PALETTE.as_usize() as *mut u8,
            SHARED_PALETTE.0.len(),
        );
        let colors: [gba::video::Color; 16] = [
            crate::color::TRANSPARENT, // Can't be accessed by the mapping function being used
            crate::color::RED,
            crate::color::GREEN,
            crate::color::BLUE,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
            crate::color::TRANSPARENT,
        ];
        copy_nonoverlapping(
            colors.as_ptr(),
            BG_PALETTE.index(16 * 15).as_usize() as *mut Color,
            colors.len(),
        );
        // Cga8x8Thick.bitunpack_8bpp(CHARBLOCK1_8BPP.as_region(), 0);
    }
}
