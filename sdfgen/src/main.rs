use ab_glyph::{Font, FontVec, OutlineCurve, PxScale, ScaleFont};
use anyhow::Context;
use clap::{command, Arg, Command};
use image::{ColorType, GrayImage};
use std::fs;
use tracing::{error, info, instrument};

fn args() -> Command {
    command!()
        .arg(
            Arg::new("font")
                .help("Font to generate SDF atlas from")
                .required(true),
        )
        .arg(
            Arg::new("atlas")
                .short('a')
                .long("atlas")
                .help("List of characters to include in atlas"),
        )
}

#[instrument(err)]
fn load_font(path: &str) -> anyhow::Result<FontVec> {
    let bytes = fs::read(path)?;
    FontVec::try_from_vec(bytes).context("Failed parsing font")
}

fn ascii_table() -> String {
    let mut table = String::new();
    for i in 0..127 {
        table.push(i as u8 as char);
    }
    table
}

struct RenderedGlyph {
    bounds: Rect,
    buffer: Vec<u8>,
}

#[derive(Debug)]
struct Rect {
    pos: (u32, u32),
    width: u32,
    height: u32,
}

struct Atlas {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl Atlas {
    pub fn new(glyphs: &[RenderedGlyph]) -> Self {
        let d = (glyphs.len() as f32).sqrt().ceil() as usize;

        // get bounds of all glyphs
        let mut width = 0;
        let mut height = 0;

        for y in 0..d {
            let mut row_width = 0;
            for x in 0..d {
                let idx = (y * d) + x;
                let Some(g) = glyphs.get(idx) else { continue };
                //row_width += g.bounds.width as usize;
                row_width += 16;
            }
            width = width.max(row_width);
            height += 16;
        }

        let mut buffer = vec![0; width * height];
        // now draw glyphs into place
        let mut cursor = (0, 0);
        for y in 0..d {
            for x in 0..d {
                let idx = (y * d) + x;
                let Some(g) = glyphs.get(idx) else { continue };
                println!("{:?}", g.bounds);
                // draw at cursor
                for y in 0..g.bounds.height {
                    for x in 0..g.bounds.width {
                        let src_idx = (y * g.bounds.width) + x;

                        let dst_coord = (x + cursor.0, y + cursor.1);
                        let dst_idx = dst_coord.1 * width as u32 + dst_coord.0;
                        buffer[dst_idx as usize] = g.buffer[src_idx as usize];
                    }
                }
                //cursor.0 += g.bounds.width;
                cursor.0 += 16;
            }
            cursor.1 += 16;
            cursor.0 = 0;
        }

        Self {
            width: width as u32,
            height: height as u32,
            buffer,
        }
    }
}

#[instrument(skip(font), err)]
fn render_glyph(font: &impl Font, glyph: char) -> anyhow::Result<RenderedGlyph> {
    let font = font.as_scaled(PxScale { x: 16.0, y: 16.0 });
    // first maybe just blit the image
    let glyph = font.scaled_glyph(glyph);
    let Some(outline) = font.outline_glyph(glyph) else {
        return Ok(RenderedGlyph {
            bounds: Rect {
                pos: (0, 0),
                width: 16,
                height: 16,
            },
            buffer: vec![0; 16 * 16],
        });
    };
    let bounds = dbg!(outline.px_bounds());
    let min = (bounds.min.x as i32, bounds.min.y as i32);
    let max = (bounds.max.x as i32, bounds.max.y as i32);
    let width = max.0 - min.0;
    let height = max.1 - min.1;

    let mut buffer = vec![0; width as usize * height as usize];

    outline.draw(|x, y, c| {
        let idx = (y * width as u32) + x;
        buffer[idx as usize] = (c * 256.0) as u8;
    });

    Ok(RenderedGlyph {
        bounds: Rect {
            pos: ((min.0 + 16) as u32, (min.1 + 16) as u32),
            width: width as u32,
            height: height as u32,
        },
        buffer,
    })
}

fn run() -> anyhow::Result<()> {
    let args = args().get_matches();

    let font: &String = args
        .get_one("font")
        .context("Failed to find argument 'font'")?;
    let font = load_font(font)?;

    let ascii = ascii_table();
    let table: Vec<char> = args.get_one("atlas").unwrap_or(&ascii).chars().collect();

    // break atlas into a roughly square grid
    info!("Generating atlas:");
    let d = (table.len() as f32).sqrt().ceil() as usize;
    let mut atlas = vec![];
    for y in 0..d {
        for x in 0..d {
            let index = (d * y) + x;
            if index < table.len() {
                let c = table[index];
                atlas.push(c);

                let esc: String = c
                    .escape_debug()
                    .filter(|c| *c != '{' && *c != '}' && *c != 'u')
                    .collect();
                print!("{esc:5}");
            }
        }
        println!();
    }

    let atlas: Vec<RenderedGlyph> = atlas
        .iter()
        .map(|c| render_glyph(&font, *c).unwrap())
        .collect();
    let atlas = Atlas::new(&atlas);

    image::save_buffer(
        "out.bmp",
        &atlas.buffer,
        atlas.width,
        atlas.height,
        ColorType::L8,
    )?;

    Ok(())
}

fn main() {
    rivik::console::init();
    if let Err(e) = run() {
        error!("{e}");
    }
}
