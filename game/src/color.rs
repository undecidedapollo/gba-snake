use gba::video::Color;

pub const fn make_color(r: u16, g: u16, b: u16) -> Color {
    return Color(r | (g << 5) | (b << 10));
}

pub const TRANSPARENT: Color = make_color(0, 0, 0);
pub const RED: Color = make_color(0b11111, 0, 0);
pub const GREEN: Color = make_color(0, 0b11111, 0);
pub const BLUE: Color = make_color(0, 0, 0b11111);
