#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use snake::{
    assets::{AssetBgTile, FRUIT_BANANA, FRUIT_CHERRY, SHARED_PALETTE},
    fruit::FruitManager,
    keys::FRAME_KEYS,
    logger,
    math::{Powers, divisible_by_num},
    rng::{self},
    snake::Snake,
};

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // #[cfg(debug_assertions)]
    if let Ok(mut logger) = MgbaBufferedLogger::try_new(MgbaMessageLevel::Fatal) {
        writeln!(logger, "{info}").ok();
    }
    loop {}
}

#[unsafe(link_section = ".iwram")]
extern "C" fn irq_handler(b: IrqBits) {
    if b.vblank() {
        // We'll read the keys during vblank and store it for later.
        FRAME_KEYS.write(KEYINPUT.read());
    }
}

fn reset_data_for_game() {
    let mut ottr = ObjAttr::new();
    ottr.0 = ottr.0.with_style(ObjDisplayStyle::NotDisplayed);
    OBJ_ATTR_ALL.iter().for_each(|va| va.write(ottr));

    let zeros: [u32; 32] = core::array::repeat(0);

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
    }
}

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    logger::init_logger();

    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    VBlankIntrWait();

    let no_display = ObjAttr0::new().with_style(ObjDisplayStyle::NotDisplayed);
    OBJ_ATTR0.iter().for_each(|va| va.write(no_display));

    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_0)
            .with_obj_vram_1d(true)
            .with_show_bg0(true)
            .with_show_obj(true),
    );
    BG0CNT.write(
        BackgroundControl::new()
            .with_size(0)
            .with_screenblock(1)
            .with_bpp8(true)
            .with_charblock(0),
    );

    reset_data_for_game();
    let snake = Snake::init();
    let fruit = FruitManager::init();
    fruit.spawn_fruit();
    fruit.spawn_fruit();

    let mut loop_counter: u16 = 0;
    let mask: u16 = (1 << Powers::_256.as_u16()) - 1;
    let next_num = || (rng::next_u16() & mask) + 64;
    let mut next_spawn: u16 = next_num();

    loop {
        VBlankIntrWait();
        loop_counter = loop_counter.wrapping_add(1);
        next_spawn = next_spawn.saturating_sub(1);
        snake.tick(fruit);
        if next_spawn == 0 {
            fruit.spawn_fruit();
            next_spawn = next_num();
        }
    }
}
