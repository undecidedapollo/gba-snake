#![no_std]
#![no_main]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use game_of_life::logger;
use gba::prelude::*;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    #[cfg(debug_assertions)]
    if let Ok(mut logger) = MgbaBufferedLogger::try_new(MgbaMessageLevel::Fatal) {
        writeln!(logger, "{info}").ok();
    }
    loop {}
}

// #[allow(dead_code)]
// const FOO: Align4<[u8; 14]> = include_aligned_bytes!("foo.txt");

#[unsafe(link_section = ".ewram")]
static FRAME_KEYS: GbaCell<KeyInput> = GbaCell::new(KeyInput::new());

#[unsafe(link_section = ".iwram")]
extern "C" fn irq_handler(b: IrqBits) {
    if b.vblank() {
        // We'll read the keys during vblank and store it for later.
        FRAME_KEYS.write(KEYINPUT.read());
    }
}

static TEST_TILE: gba::Align4<[u8; 256]> = include_aligned_bytes!("../../../asset_out/bob.sprite");

static TEST_PALETTE: gba::Align4<[u8; 16]> =
    include_aligned_bytes!("../../../asset_out/shared.palette");

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
            .with_show_obj(true),
    );

    unsafe {
        copy_nonoverlapping(
            TEST_PALETTE.0.as_ptr(),
            OBJ_PALETTE.as_usize() as *mut u8,
            TEST_PALETTE.0.len(),
        );

        copy_nonoverlapping(
            TEST_TILE.0.as_ptr(),
            OBJ_TILES.index(0).as_usize() as *mut u8,
            TEST_TILE.0.len(),
        );
    }

    let mut obj = ObjAttr::new();
    obj.0 = obj
        .0
        .with_style(ObjDisplayStyle::Normal)
        .with_shape(ObjShape::Square)
        .with_bpp8(true)
        .with_y(50);
    obj.1 = obj.1.with_size(1).with_x(100);
    obj.2 = obj.2.with_tile_id(0);

    OBJ_ATTR_ALL.index(0).write(obj);

    let mut x = 100i16;
    let mut y = 50i16;

    loop {
        VBlankIntrWait();

        let k = FRAME_KEYS.read();
        if k.left() {
            x = x.wrapping_sub(1);
        }
        if k.right() {
            x = x.wrapping_add(1);
        }
        if k.up() {
            y = y.wrapping_sub(1);
        }
        if k.down() {
            y = y.wrapping_add(1);
        }

        // Clamp/wrap to screen if you like:
        x = ((x as i32 + 512) % 512) as i16; // OAM x wraps in 9 bits
        y = ((y as i32 + 256) % 256) as i16; // OAM y wraps in 8 bits

        let mut cur = OBJ_ATTR_ALL.index(0).read();
        cur.0 = cur.0.with_y(y as u16);
        cur.1 = cur.1.with_x(x as u16);
        OBJ_ATTR_ALL.index(0).write(cur);
    }
}
