#![no_std]
#![no_main]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use snake::{gba_warning, logger};

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

static SNAKE_HEAD_UP: gba::Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_up.sprite");
static SNAKE_HEAD_LEFT: gba::Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_left.sprite");
static SNAKE_HEAD_RIGHT: gba::Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_right.sprite");
static SNAKE_HEAD_DOWN: gba::Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/head_down.sprite");

static SNAKE_HEAD_PALETTE: gba::Align4<[u8; 16]> =
    include_aligned_bytes!("../../asset_out/shared.palette");

#[repr(u8)]
enum SnakeAnimationTile {
    Up = 0,
    Left = 2,
    Right = 4,
    Down = 6,
}

impl Into<usize> for SnakeAnimationTile {
    fn into(self) -> usize {
        return self as usize;
    }
}
impl Into<u16> for SnakeAnimationTile {
    fn into(self) -> u16 {
        return self as u16;
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
            .with_show_obj(true),
    );

    unsafe {
        copy_nonoverlapping(
            SNAKE_HEAD_PALETTE.0.as_ptr(),
            OBJ_PALETTE.as_usize() as *mut u8,
            SNAKE_HEAD_PALETTE.0.len(),
        );

        copy_nonoverlapping(
            SNAKE_HEAD_UP.0.as_ptr(),
            OBJ_TILES.index(SnakeAnimationTile::Up.into()).as_usize() as *mut u8,
            SNAKE_HEAD_UP.0.len(),
        );
        copy_nonoverlapping(
            SNAKE_HEAD_LEFT.0.as_ptr(),
            OBJ_TILES.index(SnakeAnimationTile::Left.into()).as_usize() as *mut u8,
            SNAKE_HEAD_LEFT.0.len(),
        );
        copy_nonoverlapping(
            SNAKE_HEAD_RIGHT.0.as_ptr(),
            OBJ_TILES.index(SnakeAnimationTile::Right.into()).as_usize() as *mut u8,
            SNAKE_HEAD_RIGHT.0.len(),
        );
        copy_nonoverlapping(
            SNAKE_HEAD_DOWN.0.as_ptr(),
            OBJ_TILES.index(SnakeAnimationTile::Down.into()).as_usize() as *mut u8,
            SNAKE_HEAD_DOWN.0.len(),
        );
    }

    let mut obj = ObjAttr::new();
    obj.0 = obj
        .0
        .with_style(ObjDisplayStyle::Normal)
        .with_shape(ObjShape::Square)
        .with_bpp8(true)
        .with_y(50);
    obj.1 = obj.1.with_size(0).with_x(100);
    obj.2 = obj.2.with_tile_id(0);

    OBJ_ATTR_ALL.index(0).write(obj);

    let mut x: i16 = 100;
    let mut y: i8 = 50;

    loop {
        VBlankIntrWait();

        let mut print = false;
        let mut new_tile_id = None;
        let k = FRAME_KEYS.read();
        if k.left() {
            x = x.wrapping_sub(1);
            print = true;
            new_tile_id = Some(SnakeAnimationTile::Left);
        } else if k.right() {
            x = x.wrapping_add(1);
            print = true;
            new_tile_id = Some(SnakeAnimationTile::Right);
        } else if k.up() {
            y = y.wrapping_sub(1);
            print = true;
            new_tile_id = Some(SnakeAnimationTile::Up);
        } else if k.down() {
            y = y.wrapping_add(1);
            print = true;
            new_tile_id = Some(SnakeAnimationTile::Down);
        }

        // Clamp/wrap to screen if you like:
        // x = (x + 256) % 512; // OAM x wraps in 9 bits
        // y = (y + 128) % 256; // OAM y wraps in 8 bits

        x = mask_signed_x(x);

        if print {
            gba_warning!("x:{x} y:{y}");
        }

        let mut cur = OBJ_ATTR_ALL.index(0).read();
        cur.set_x(x as u16);
        cur.set_y(y as u16);
        if let Some(new_tile_id) = new_tile_id {
            cur.set_tile_id(new_tile_id.into());
        }
        OBJ_ATTR_ALL.index(0).write(cur);
    }
}

fn mask_signed_x(x: i16) -> i16 {
    let v = x & 0x1FF; // keep 9 bits (0..511)
    v - ((v & 0x100) << 1)
}
