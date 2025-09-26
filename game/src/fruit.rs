use core::ptr::copy_nonoverlapping;

use gba::mmio::{CHARBLOCK0_8BPP, TEXT_SCREENBLOCKS};

use crate::{
    assets::{AssetBgTile, FRUIT_BANANA, FRUIT_CHERRY},
    rng::{self},
};

pub struct FruitManager {
    fruit_locs: [Option<(u16, u16)>; 5],
}

#[unsafe(link_section = ".ewram")]
pub static mut FRUIT_MANAGER_STORAGE: FruitManager = FruitManager {
    fruit_locs: [None; 5],
};

impl FruitManager {
    pub fn init() -> &'static mut Self {
        let fruit = unsafe { &mut (*core::ptr::addr_of_mut!(FRUIT_MANAGER_STORAGE)) };

        unsafe {
            copy_nonoverlapping(
                FRUIT_CHERRY.0.as_ptr(),
                CHARBLOCK0_8BPP.index(AssetBgTile::Cherry.into()).as_usize() as *mut u8,
                FRUIT_CHERRY.0.len(),
            );
            copy_nonoverlapping(
                FRUIT_BANANA.0.as_ptr(),
                CHARBLOCK0_8BPP.index(AssetBgTile::Banana.into()).as_usize() as *mut u8,
                FRUIT_BANANA.0.len(),
            );
        }

        fruit
    }

    pub fn reset(&mut self) {
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
            }
        }
    }

    pub fn try_eat_fruit(&mut self, loc: (u16, u16)) -> Option<()> {
        let Some((idx, _)) = self.fruit_locs.iter().enumerate().find(|x| {
            let Some((x, y)) = x.1 else {
                return false;
            };

            return *x == loc.0 && *y == loc.1;
        }) else {
            return None;
        };

        self.fruit_locs[idx] = None;
        let new_idx = TEXT_SCREENBLOCKS
            .get_frame(1)
            .unwrap()
            .get(loc.0 as usize, loc.1 as usize)
            .unwrap();
        new_idx.write(new_idx.read().with_tile(AssetBgTile::Blank.into()));

        Some(())
    }

    pub fn spawn_fruit(&mut self) {
        let Some((blank_idx, _)) = self.fruit_locs.iter().enumerate().find(|x| x.1.is_none())
        else {
            return;
        };

        self.spawn_fruit_at_idx(blank_idx);
    }

    fn spawn_fruit_at_idx(&mut self, idx: usize) {
        let x = (rng::next_u32() >> 8) as usize & 0b0000_1111;
        let y = (rng::next_u32() >> 8) as usize & 0b0000_1111;
        let tile = if rng::next_bool() {
            AssetBgTile::Cherry
        } else {
            AssetBgTile::Banana
        };
        let new_idx = TEXT_SCREENBLOCKS.get_frame(1).unwrap().get(x, y).unwrap();
        new_idx.write(new_idx.read().with_tile(tile.into()));
        self.fruit_locs[idx] = Some((x as u16, y as u16));
    }
}
