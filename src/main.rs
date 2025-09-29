#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::fmt::Write;
use gba::prelude::*;
use snake::{
    assets::{self},
    color::PaletteColor,
    fruit::FruitManager,
    keys::FRAME_KEYS,
    logger,
    score::ScoreManager,
    screen_text::ScreenTextManager,
    snake::{Snake, SnakeTickResponse},
};

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
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

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    logger::init_logger();

    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    VBlankIntrWait();

    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_0)
            .with_obj_vram_1d(true)
            .with_show_bg0(true)
            .with_show_bg1(true)
            .with_show_obj(true),
    );
    BG0CNT.write(
        BackgroundControl::new()
            .with_size(0)
            .with_screenblock(1)
            .with_bpp8(true)
            .with_charblock(0),
    );
    BG1CNT.write(
        BackgroundControl::new()
            .with_size(0)
            .with_screenblock(2)
            .with_bpp8(false)
            .with_charblock(1),
    );

    assets::reset_data();

    let snake = Snake::init();
    let fruit = FruitManager::init();

    let mut loop_counter: u16 = 0;

    loop {
        VBlankIntrWait();
        loop_counter = loop_counter.wrapping_add(1);
        match snake.tick(fruit) {
            SnakeTickResponse::None => {}
            SnakeTickResponse::GameOver => {
                assets::reset_data();
                ScoreManager::reset_w_score();

                for _ in 0..5 {
                    VBlankIntrWait();
                }

                let mut go_text =
                    ScreenTextManager::write_text(2, "GAME OVER", (5, 6), PaletteColor::Red, false);

                for _ in 0..120 {
                    VBlankIntrWait();
                }
                loop_counter = 0;
                go_text.take();

                ScreenTextManager::unlock_all();
                assets::reset_data();
                snake.reset();
                fruit.reset();

                VBlankIntrWait();
                VBlankIntrWait();
                ScoreManager::reset(0);
                continue;
            }
        }
        fruit.tick();
        ScoreManager::tick();
    }
}
