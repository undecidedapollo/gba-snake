use core::ptr::copy_nonoverlapping;

use gba::{Align4, prelude::*};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    assets::{
        AssetObjTile, SNAKE_BODY_1, SNAKE_HEAD_DOWN, SNAKE_HEAD_LEFT, SNAKE_HEAD_RIGHT,
        SNAKE_HEAD_UP,
    },
    fruit::FruitManager,
    gba_warning,
    keys::FRAME_KEYS,
    math::{Powers, divisible_by_num},
};

#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum MovementTwoBit {
    Up = 0,
    Left = 1,
    Right = 2,
    Down = 3,
    Stall = 4,
}

fn mask_signed_x(x: i16) -> i16 {
    let v = x & 0x1FF; // keep 9 bits (0..511)
    v - ((v & 0x100) << 1)
}

pub struct Snake {
    len: usize,
    speed: u16,
    bonus_movement_counter: u16,
    next_dir: AssetObjTile,
    should_spawn_segment: bool,
    segments: [Align4<ObjAttr>; 64],
}

#[unsafe(link_section = ".ewram")]
static mut SNAKE_STORAGE: Snake = Snake {
    len: 6,
    speed: 8,
    bonus_movement_counter: 0,
    next_dir: AssetObjTile::SnakeHeadUp,
    should_spawn_segment: false,
    segments: [const { Align4(ObjAttr::new()) }; 64],
};

impl Snake {
    pub fn init() -> &'static mut Self {
        let snake = unsafe { &mut (*core::ptr::addr_of_mut!(SNAKE_STORAGE)) };

        unsafe {
            copy_nonoverlapping(
                SNAKE_HEAD_UP.0.as_ptr(),
                OBJ_TILES.index(AssetObjTile::SnakeHeadUp.into()).as_usize() as *mut u8,
                SNAKE_HEAD_UP.0.len(),
            );
            copy_nonoverlapping(
                SNAKE_HEAD_LEFT.0.as_ptr(),
                OBJ_TILES
                    .index(AssetObjTile::SnakeHeadLeft.into())
                    .as_usize() as *mut u8,
                SNAKE_HEAD_LEFT.0.len(),
            );
            copy_nonoverlapping(
                SNAKE_HEAD_RIGHT.0.as_ptr(),
                OBJ_TILES
                    .index(AssetObjTile::SnakeHeadRight.into())
                    .as_usize() as *mut u8,
                SNAKE_HEAD_RIGHT.0.len(),
            );
            copy_nonoverlapping(
                SNAKE_HEAD_DOWN.0.as_ptr(),
                OBJ_TILES
                    .index(AssetObjTile::SnakeHeadDown.into())
                    .as_usize() as *mut u8,
                SNAKE_HEAD_DOWN.0.len(),
            );
            copy_nonoverlapping(
                SNAKE_BODY_1.0.as_ptr(),
                OBJ_TILES.index(AssetObjTile::SnakeBody1.into()).as_usize() as *mut u8,
                SNAKE_BODY_1.0.len(),
            );
        }

        snake.reset();
        snake
    }

    pub fn reset(&mut self) {
        self.len = 1;
        self.speed = 12;
        self.bonus_movement_counter = 0;
        self.next_dir = AssetObjTile::SnakeHeadUp;

        self.segments[0] =
            Snake::init_snake_tile(AssetObjTile::SnakeHeadUp, 6, 6, MovementTwoBit::Up);
        self.create_new_segment(None);
        self.create_new_segment(None);
    }

    fn create_new_segment(&mut self, coords: Option<(u16, u16)>) {
        let (x, y) = if let Some((x, y)) = coords {
            (x, y)
        } else {
            let last = self.segments[self.len - 1].0;
            (last.1.x(), last.0.y())
        };

        let x_div = (x >> 3) as i8;
        let y_div = (y >> 3) as i8;

        self.segments[self.len] = Snake::init_snake_tile(
            AssetObjTile::SnakeBody1,
            (x_div) as u8,
            (y_div) as u8,
            MovementTwoBit::Stall,
        );
        self.len += 1;
        self.should_spawn_segment = false;
    }

    fn init_snake_tile(
        tile_id: AssetObjTile,
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

    pub fn tick(&mut self, fruit: &mut FruitManager) {
        let k = FRAME_KEYS.read();
        let mut head = self.segments[0].0;

        if k.left() {
            self.next_dir = AssetObjTile::SnakeHeadLeft;
        } else if k.right() {
            self.next_dir = AssetObjTile::SnakeHeadRight;
        } else if k.up() {
            self.next_dir = AssetObjTile::SnakeHeadUp;
        } else if k.down() {
            self.next_dir = AssetObjTile::SnakeHeadDown;
        }

        if divisible_by_num(head.1.x(), Powers::_8) && divisible_by_num(head.0.y(), Powers::_8) {
            head.set_tile_id(self.next_dir.into());
        }

        let mut num_iterations = self.speed >> 4;
        self.bonus_movement_counter += self.speed & 0xF;

        if self.bonus_movement_counter >= 16 {
            self.bonus_movement_counter = self.bonus_movement_counter - 16;
            num_iterations += 1;
        }
        gba_warning!("{}", num_iterations);

        let mut prev_x = head.1.x();
        let mut prev_y = head.0.y() as u16;

        let mut last_good_spawn: Option<(u16, u16)> = None;

        for sprite_idx in 0..self.len {
            let mut obj = if sprite_idx == 0 {
                head
            } else {
                self.segments[sprite_idx].0
            };
            let obj_tile: AssetObjTile =
                AssetObjTile::try_from(obj.2.tile_id() as u8).unwrap_or(AssetObjTile::SnakeHeadUp);
            let mut x = obj.1.x();
            let mut y = obj.0.y() as i8;

            for iter_num in 0..num_iterations {
                match obj_tile {
                    AssetObjTile::SnakeHeadUp => {
                        y = y.wrapping_sub(1);
                    }
                    AssetObjTile::SnakeHeadDown => {
                        y = y.wrapping_add(1);
                    }
                    AssetObjTile::SnakeHeadLeft => {
                        x = x.wrapping_sub(1);
                    }
                    AssetObjTile::SnakeHeadRight => {
                        x = x.wrapping_add(1);
                    }
                    AssetObjTile::SnakeBody1 => {
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
                            MovementTwoBit::Stall => {}
                        }
                    }
                };
                x = mask_signed_x(x as i16) as u16;
                let y_test = y as u16;
                let x_is_div = divisible_by_num(x, Powers::_8);
                let y_is_div = divisible_by_num(y_test, Powers::_8);
                if sprite_idx != 0 && x_is_div && y_is_div {
                    let movement_dif =
                        MovementTwoBit::try_from(obj.2.palbank()).unwrap_or(MovementTwoBit::Up);

                    let to_update = match movement_dif {
                        MovementTwoBit::Up
                        | MovementTwoBit::Down
                        | MovementTwoBit::Left
                        | MovementTwoBit::Right => {
                            if prev_x < x {
                                Some(MovementTwoBit::Left)
                            } else if prev_x > x {
                                Some(MovementTwoBit::Right)
                            } else if prev_y < y_test {
                                Some(MovementTwoBit::Up)
                            } else if prev_y > y_test {
                                Some(MovementTwoBit::Down)
                            } else {
                                gba_warning!("Same coordinates error");
                                None
                            }
                        }
                        MovementTwoBit::Stall => {
                            if prev_x < x && x - prev_x >= 8 {
                                Some(MovementTwoBit::Left)
                            } else if prev_x > x && prev_x - x >= 8 {
                                Some(MovementTwoBit::Right)
                            } else if prev_y < y_test && y_test - prev_y >= 8 {
                                Some(MovementTwoBit::Up)
                            } else if prev_y > y_test && prev_y - y_test >= 8 {
                                Some(MovementTwoBit::Down)
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(new_move_dir) = to_update {
                        obj.set_palbank(new_move_dir.into());
                    }

                    // It is the last segment and the last time it is going to hit a good spot.
                    if self.should_spawn_segment
                        && sprite_idx == self.len - 1
                        && num_iterations - iter_num < 8
                    {
                        last_good_spawn = Some((x, y_test));
                        self.should_spawn_segment = false;
                    }
                } else if sprite_idx == 0 && x_is_div && y_is_div {
                    let x_div = x >> 3;
                    let y_div = (y as u8) >> 3;
                    gba_warning!("{x_div} {y_div}");

                    if let Some(_) = fruit.try_eat_fruit((x_div, y_div as u16)) {
                        self.speed += 1;
                        self.should_spawn_segment = true;
                    }
                }
            }
            obj.set_x(x);
            obj.set_y(y as u16);
            prev_x = x;
            prev_y = y as u16;
            self.segments[sprite_idx] = Align4(obj);
            // gba_warning!("{sprite_idx} {} {} {}", x, y, num_iterations);
        }

        if last_good_spawn.is_some() {
            self.create_new_segment(last_good_spawn);
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
