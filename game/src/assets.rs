use gba::{Align4, include_aligned_bytes};
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
