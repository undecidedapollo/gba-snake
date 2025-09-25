#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
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
static SNAKE_BODY_1: gba::Align4<[u8; 64]> =
    include_aligned_bytes!("../../asset_out/body_1.sprite");

static SNAKE_HEAD_PALETTE: gba::Align4<[u8; 16]> =
    include_aligned_bytes!("../../asset_out/shared.palette");

#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
enum SnakeAnimationTile {
    Up = 0,
    Left = 2,
    Right = 4,
    Down = 6,
    Body1 = 8,
}

#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u16)]
enum MovementTwoBit {
    Up = 0,
    Left = 1,
    Right = 2,
    Down = 3,
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

fn init_snake_tile(
    tile_id: SnakeAnimationTile,
    grid_x: u8,
    grid_y: u8,
    dir: MovementTwoBit,
) -> Align4<ObjAttr> {
    let mut obj = ObjAttr::new();
    obj.0 = obj
        .0
        .with_style(ObjDisplayStyle::Normal)
        .with_shape(ObjShape::Square)
        .with_bpp8(true)
        .with_y((grid_y as u16) * 8);
    obj.1 = obj.1.with_size(0).with_x((grid_x as u16) * 8);
    obj.2 = obj.2.with_tile_id(tile_id.into()).with_palbank(dir.into());
    Align4(obj)
}

struct Snake {
    len: usize,
    speed: u8,
    bonus_movement_counter: u8,
    next_dir: SnakeAnimationTile,
    segments: [Align4<ObjAttr>; 64],
}

#[unsafe(link_section = ".ewram")]
static mut SNAKE_STORAGE: Snake = Snake {
    len: 6,
    speed: 8,
    bonus_movement_counter: 0,
    next_dir: SnakeAnimationTile::Up,
    segments: [const { Align4(ObjAttr::new()) }; 64],
};

// fn get_obj(idx: usize) -> ObjAttr {
//     return OBJ_ATTR_ALL.index(idx).read();
// }

// fn write_obj(idx: usize, obj: ObjAttr) {
//     OBJ_ATTR_ALL.index(idx).write(obj);
// }

fn divisible_by_8(x: u16) -> bool {
    (x & 0b111) == 0
}

impl Snake {
    fn init() -> &'static mut Self {
        let snake = unsafe { &mut (*core::ptr::addr_of_mut!(SNAKE_STORAGE)) };
        snake.segments[0] = init_snake_tile(SnakeAnimationTile::Up, 6, 6, MovementTwoBit::Up);
        snake.segments[1] = init_snake_tile(SnakeAnimationTile::Body1, 6, 7, MovementTwoBit::Up);
        snake.segments[2] = init_snake_tile(SnakeAnimationTile::Body1, 6, 8, MovementTwoBit::Up);
        snake.segments[3] = init_snake_tile(SnakeAnimationTile::Body1, 6, 9, MovementTwoBit::Up);
        snake.segments[4] = init_snake_tile(SnakeAnimationTile::Body1, 6, 10, MovementTwoBit::Up);
        snake.segments[5] = init_snake_tile(SnakeAnimationTile::Body1, 6, 11, MovementTwoBit::Up);
        snake.len = 6;
        snake.speed = 8;
        snake.bonus_movement_counter = 0;
        snake.next_dir = SnakeAnimationTile::Up;
        snake
    }

    fn tick(&mut self) {
        let k = FRAME_KEYS.read();
        let mut head = self.segments[0].0;

        if k.left() {
            self.next_dir = SnakeAnimationTile::Left;
        } else if k.right() {
            self.next_dir = SnakeAnimationTile::Right;
        } else if k.up() {
            self.next_dir = SnakeAnimationTile::Up;
        } else if k.down() {
            self.next_dir = SnakeAnimationTile::Down;
        }

        if divisible_by_8(head.1.x()) && divisible_by_8(head.0.y()) {
            head.set_tile_id(self.next_dir.into());
        }

        let mut num_iterations = self.speed >> 4;
        self.bonus_movement_counter += self.speed & 0xF;
        if self.bonus_movement_counter >= 16 {
            self.bonus_movement_counter = 0;
            num_iterations += 1;
        }

        let mut prev_x = head.1.x();
        let mut prev_y = head.0.y() as i8;
        for sprite_idx in 0..self.len {
            let mut obj = if sprite_idx == 0 {
                head
            } else {
                self.segments[sprite_idx].0
            };
            let obj_tile: SnakeAnimationTile = SnakeAnimationTile::try_from(obj.2.tile_id() as u8)
                .unwrap_or(SnakeAnimationTile::Up);
            let mut x = obj.1.x();
            let mut y = obj.0.y() as i8;
            for _ in 0..num_iterations {
                match obj_tile {
                    SnakeAnimationTile::Up => {
                        // gba_warning!("up");
                        y = y.wrapping_sub(1);
                    }
                    SnakeAnimationTile::Down => {
                        // gba_warning!("down");
                        y = y.wrapping_add(1);
                    }
                    SnakeAnimationTile::Left => {
                        // gba_warning!("left");
                        x = x.wrapping_sub(1);
                    }
                    SnakeAnimationTile::Right => {
                        // gba_warning!("right");
                        x = x.wrapping_add(1);
                    }
                    SnakeAnimationTile::Body1 => {
                        let movement_dif =
                            MovementTwoBit::try_from(obj.2.palbank()).unwrap_or(MovementTwoBit::Up);
                        match movement_dif {
                            MovementTwoBit::Up => {
                                y = y.wrapping_sub(1);
                            }
                            MovementTwoBit::Down => {
                                y = y.wrapping_add(1);
                            }
                            MovementTwoBit::Left => {
                                x = x.wrapping_sub(1);
                            }
                            MovementTwoBit::Right => {
                                x = x.wrapping_add(1);
                            }
                        }
                    }
                };
                x = mask_signed_x(x as i16) as u16;
                if obj_tile == SnakeAnimationTile::Body1
                    && divisible_by_8(x)
                    && divisible_by_8(y as u16)
                {
                    let new_move_dir = if prev_x < x {
                        MovementTwoBit::Left
                    } else if prev_x > x {
                        MovementTwoBit::Right
                    } else if prev_y < y {
                        MovementTwoBit::Up
                    } else if prev_y > y {
                        MovementTwoBit::Down
                    } else {
                        gba_warning!("Same coordinates error");
                        MovementTwoBit::Up
                    };

                    obj.set_palbank(new_move_dir.into());
                }
            }
            obj.set_x(x);
            obj.set_y(y as u16);
            prev_x = x;
            prev_y = y;
            self.segments[sprite_idx] = Align4(obj);
            // gba_warning!("{sprite_idx} {} {} {}", x, y, num_iterations);
        }

        unsafe {
            copy_nonoverlapping(
                self.segments.as_ptr(),
                OBJ_ATTR_ALL.index(0).as_mut_ptr() as *mut Align4<ObjAttr>,
                self.len,
            );
        }
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
        copy_nonoverlapping(
            SNAKE_BODY_1.0.as_ptr(),
            OBJ_TILES.index(SnakeAnimationTile::Body1.into()).as_usize() as *mut u8,
            SNAKE_BODY_1.0.len(),
        );
    }

    let snake = Snake::init();

    let mut loop_counter: u16 = 0;

    loop {
        VBlankIntrWait();
        loop_counter = loop_counter.wrapping_add(1);
        if loop_counter == 0 {
            snake.speed += 1;
        }
        snake.tick();
    }
}

fn mask_signed_x(x: i16) -> i16 {
    let v = x & 0x1FF; // keep 9 bits (0..511)
    v - ((v & 0x100) << 1)
}
