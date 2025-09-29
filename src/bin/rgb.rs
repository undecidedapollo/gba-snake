#![no_std]
#![no_main]

use gba::prelude::*;
use snake::{gba_error, gba_info, logger};

const SCREEN_WIDTH: usize = 240;
const SCREEN_HEIGHT: usize = 160;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    VIDEO3_VRAM.index(0, 0).write(Color::RED);
    loop {}
}

#[unsafe(no_mangle)]
fn main() -> ! {
    logger::init_logger();

    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_3)
            .with_show_bg2(true),
    );

    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    // CPU frequency is 16.78 MHz, so a tick with a prescale of 64 is a little less than 2.5 ms
    TIMER0_CONTROL.write(
        TimerControl::new()
            .with_scale(TimerScale::_1)
            .with_enabled(true),
    );
    BACKDROP_COLOR.write(Color::RED);

    let mut row: usize = 0;
    let mut col: usize = 0;
    let mut color_idx = 0;
    let colors = [Color::RED, Color::WHITE, Color::BLUE];

    loop {
        VIDEO3_VRAM.index(col, row).write(colors[color_idx]);
        gba_error!("Position: ({}, {}), Color index: {}", col, row, color_idx);
        gba_info!("Frame drawn at {},{}", col, row);

        VBlankIntrWait();
        col = col + 16;
        if col >= SCREEN_WIDTH {
            col = 0;
            row = row + 16;
        }
        if row >= SCREEN_HEIGHT {
            col = 0;
            row = 0;
            color_idx = color_idx + 1;
        }
        if color_idx >= colors.len() {
            color_idx = 0;
        }
    }
}
