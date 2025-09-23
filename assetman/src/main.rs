use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::Write,
};

use image::{GenericImageView, ImageReader, Pixel};

fn main() -> Result<(), Box<dyn Error>> {
    // let mut bmp_from_file = BMP::new_from_file("assets/s1.bmp").unwrap();
    // let mut x = bmp_from_file.get_pixel_data().unwrap();
    // let y = x.pop_front().unwrap();
    let img = ImageReader::open("../assets/bob.png")?.decode()?;
    let (width, height) = img.dimensions();

    let mut next_palette_index: u8 = 0;
    let mut palette: HashMap<u16, u8> = HashMap::new();
    let mut bin: Vec<u8> = Vec::new();

    let mut get_palette_index = |color: u16| {
        if let Some(entry) = palette.get(&color) {
            return *entry;
        }
        let cur_idx = next_palette_index;
        next_palette_index += 1;
        palette.insert(color, cur_idx);
        cur_idx
    };

    for tile in 0..4 {
        for row in 0..8 {
            for col in 0..8 {
                let (row, col) = match tile {
                    0 => (row, col),
                    1 => (row, col + 8),
                    2 => (row + 8, col),
                    3 => (row + 8, col + 8),
                    _ => panic!("Shouldn't happen"),
                };
                let px = img.get_pixel(col, row);
                let [r8, g8, b8, a8] = px.channels() else {
                    panic!("Unknown rgba pattern");
                };
                if *a8 == 0_u8 {
                    bin.push(get_palette_index(0));
                    continue;
                }

                let r5 = (*r8 as u16) >> 3;
                let g5 = (*g8 as u16) >> 3;
                let b5 = (*b8 as u16) >> 3;
                // let r5 = (*r8 as u16) * 31 / 255;
                // let g5 = (*g8 as u16) * 31 / 255;
                // let b5 = (*b8 as u16) * 31 / 255;
                let color: u16 = (b5 << 10) | (g5 << 5) | r5;
                bin.push(get_palette_index(color));
            }
        }
    }

    drop(get_palette_index);

    let mut data: Vec<(u16, u8)> = palette.into_iter().collect();
    data.sort_by_key(|x| x.1);

    let data = data
        .into_iter()
        .flat_map(|x| x.0.to_le_bytes())
        .collect::<Vec<u8>>();

    let mut palette_file = File::create("../asset_out/bob.palette")?;
    palette_file.write_all(data.as_slice())?;

    let mut sprite_file = File::create("../asset_out/bob.sprite")?;
    sprite_file.write_all(bin.as_slice())?;

    println!("{:?} {:?} {:?} {} {}", data, bin, bin.len(), width, height);
    palette_file.flush()?;
    sprite_file.flush()?;
    Ok(())
}
