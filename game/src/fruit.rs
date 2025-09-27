use core::ptr::copy_nonoverlapping;

use gba::mmio::{CHARBLOCK0_8BPP, TEXT_SCREENBLOCKS};

use crate::{
    assets::{AssetBgTile, FRUIT_BANANA, FRUIT_CHERRY},
    math::Powers,
    rng::{self},
    score::ScoreManager,
};

pub struct FruitManager {
    fruit_locs: [Option<(u16, u16)>; 5],
    next_spawn: u16,
}

#[unsafe(link_section = ".ewram")]
pub static mut FRUIT_MANAGER_STORAGE: FruitManager = FruitManager {
    fruit_locs: [None; 5],
    next_spawn: 0,
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

    fn is_empty(&mut self) -> bool {
        self.fruit_locs.iter().find(|x| x.is_some()).is_none()
    }

    fn fruit_exists_idx(&mut self, loc: (u16, u16)) -> Option<usize> {
        let opt = self.fruit_locs.iter().enumerate().find(|x| {
            let Some((x, y)) = x.1 else {
                return false;
            };

            return *x == loc.0 && *y == loc.1;
        });
        opt.map(|x| x.0)
    }

    pub fn try_eat_fruit(&mut self, loc: (u16, u16)) -> Option<()> {
        let Some(idx) = self.fruit_exists_idx(loc) else {
            return None;
        };

        ScoreManager::add_to_score(100);

        self.fruit_locs[idx] = None;
        let new_idx = TEXT_SCREENBLOCKS
            .get_frame(1)
            .unwrap()
            .get(loc.0 as usize, loc.1 as usize)
            .unwrap();
        new_idx.write(new_idx.read().with_tile(AssetBgTile::Blank.into()));

        if self.next_spawn > 30 && self.is_empty() {
            self.next_spawn = 30;
        }

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
        let mut x: usize;
        let mut y: usize;
        loop {
            x = (rng::next_u32() >> 8) as usize & 0b0000_1111;
            y = (rng::next_u32() >> 8) as usize & 0b0000_1111;
            if let None = self.fruit_exists_idx((x as u16, y as u16)) {
                break;
            }
        }

        let tile = if rng::next_bool() {
            AssetBgTile::Cherry
        } else {
            AssetBgTile::Banana
        };
        let new_idx = TEXT_SCREENBLOCKS.get_frame(1).unwrap().get(x, y).unwrap();
        new_idx.write(new_idx.read().with_tile(tile.into()));
        self.fruit_locs[idx] = Some((x as u16, y as u16));
    }

    pub fn tick(&mut self) {
        self.next_spawn = self.next_spawn.saturating_sub(1);

        if self.next_spawn == 0 {
            self.spawn_fruit();
            let mask: u16 = (1 << Powers::_256.as_u16()) - 1;
            self.next_spawn = (rng::next_u16() & mask) + 64;
        }
    }
}
