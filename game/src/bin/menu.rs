#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]
#![feature(alloc_error_handler)]

use core::fmt::Write;
extern crate alloc;
use gba::prelude::*;
use snake::{
    assets::{self},
    color::PaletteColor,
    ewram_static,
    ewramstring::EwramString,
    keys::FRAME_KEYS,
    logger,
    screen_text::{ScreenTextManager, WriteTicket},
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

ewram_static!(pub HIGHSCORE_STR: EwramString<64> = EwramString::new());
// ewram_static!(pub STR_BUF_2: EwramString<64> = EwramString::new());
// ewram_static!(pub STR_BUF_3: EwramString<256> = EwramString::new());

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
    let screen = ScreenTextManager.init();
    // let snake = Snake::init();
    // let fruit = FruitManager::init();

    let mut loop_counter: u16 = 0;

    let str1 = HIGHSCORE_STR.init();

    screen
        .write_text(2, "Highscore:", (4, 6), PaletteColor::Red)
        .forever();

    let mut write_tk: Option<WriteTicket> = None;

    loop {
        VBlankIntrWait();
        if loop_counter % 60 == 0 {
            write_tk.take();
            str1.clear();
            write!(str1, "{loop_counter}").unwrap();
            write_tk = Some(screen.write_text(2, str1.as_str(), (4, 7), PaletteColor::DarkGreen));
        }
        loop_counter = loop_counter.wrapping_add(1);
    }
}
