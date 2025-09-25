use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

use image::{GenericImageView, ImageReader, Pixel};

fn main() -> Result<(), Box<dyn Error>> {
    let assets_dir = Path::new("../assets");
    let output_dir = Path::new("../asset_out");

    // Ensure output directory exists
    fs::create_dir_all(output_dir)?;

    // Shared palette across all sprites
    let mut next_palette_index: u8 = 0;
    let mut palette: HashMap<u16, u8> = HashMap::new();

    let mut get_palette_index = |color: u16| -> u8 {
        if let Some(entry) = palette.get(&color) {
            return *entry;
        }
        let cur_idx = next_palette_index;
        next_palette_index += 1;
        palette.insert(color, cur_idx);
        cur_idx
    };

    // Process all PNG files in assets directory
    let mut entries = fs::read_dir(assets_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename")?;

        println!("Processing: {}", file_stem);

        let img = ImageReader::open(&path)?.decode()?;
        let (width, height) = img.dimensions();

        let wm = width % 8;
        let hm = height % 8;
        if wm != 0 {
            return Err(format!("Expected width to be a multiple of 8 for {}, got {}", file_stem, width).into());
        }
        if hm != 0 {
            return Err(format!("Expected height to be a multiple of 8 for {}, got {}", file_stem, height).into());
        }

        let wd = width / 8;
        let hd = height / 8;
        let tiles_count = hd * wd;

        let mut bin: Vec<u8> = Vec::new();

        for tile in 0..tiles_count {
            for row in 0..8 {
                for col in 0..8 {
                    let col_mod = (tile % wd) * 8;
                    let row_mod = (tile / wd) * 8;

                    let col = col + col_mod;
                    let row = row + row_mod;

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
                    let color: u16 = (b5 << 10) | (g5 << 5) | r5;
                    bin.push(get_palette_index(color));
                }
            }
        }

        // Write sprite file for this image
        let sprite_path = output_dir.join(format!("{}.sprite", file_stem));
        let mut sprite_file = File::create(sprite_path)?;
        sprite_file.write_all(bin.as_slice())?;
        sprite_file.flush()?;

        println!("  Sprite: {}.sprite ({}x{}, {} bytes)", file_stem, width, height, bin.len());
    }

    // Write shared palette file after processing all images
    let mut palette_data: Vec<(u16, u8)> = palette.into_iter().collect();
    palette_data.sort_by_key(|x| x.1);

    let palette_bytes = palette_data
        .into_iter()
        .flat_map(|x| x.0.to_le_bytes())
        .collect::<Vec<u8>>();

    let palette_path = output_dir.join("shared.palette");
    let mut palette_file = File::create(palette_path)?;
    palette_file.write_all(palette_bytes.as_slice())?;
    palette_file.flush()?;

    println!("\nShared palette: shared.palette ({} colors)", next_palette_index);

    Ok(())
}
