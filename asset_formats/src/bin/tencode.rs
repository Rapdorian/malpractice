use asset_formats::bcn::{bc4, bc5, bc7};
use asset_formats::dds::dx10::Dx10Header;
use asset_formats::dds::{dx10, DdsHeader, FullDdsHeader};
use asset_formats::ImageFormat;
use clap::{arg, Parser};
use env_logger::Env;
use image::{open, DynamicImage, EncodableLayout};
use log::{info, warn};
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input image file
    file: String,

    /// DDS output file
    #[arg(short, long)]
    out: Option<String>,

    /// Enable Red channel
    #[arg(short)]
    red: bool,
    /// Enable Green channel
    #[arg(short)]
    green: bool,
    /// Enable Blue Channel
    #[arg(short)]
    blue: bool,
    /// Enable Alpha Channel
    #[arg(short)]
    alpha: bool,

    /// Enable Luma (grayscale) Channel
    #[arg(short, conflicts_with_all = ["red", "green", "blue"])]
    luma: bool,

    /// Enable compression
    #[arg(short)]
    compress: bool,
}

#[derive(Debug)]
struct Dds {
    magic: u32,
    header: DdsHeader,
    dx10_header: Option<Dx10Header>,
    data: Vec<Vec<u8>>,
}

fn encode_dds(data: &[DynamicImage], format: ImageFormat) -> Dds {
    if data.len() != 1 {
        panic!("unsupported number of images added to file: {}", data.len());
    }
    let header = FullDdsHeader::new(data[0].width(), data[0].height(), None, format);
    let mut array = vec![];

    for img in data {
        // convert byte format
        let bytes = match format {
            ImageFormat::Rgb8 => img.to_rgb8().to_vec(),
            ImageFormat::Rgba8 => img.to_rgba8().to_vec(),
            ImageFormat::Luma8 => img.to_luma8().to_vec(),
            ImageFormat::LumaAlpha8 => img.to_luma_alpha8().to_vec(),
            ImageFormat::Luma8_Bc4 => bytemuck::cast_slice(&bc4::encode(&img)).to_vec(),
            ImageFormat::LumaAlpha8_Bc5 => {
                bytemuck::cast_slice((&bc5::encode_grayscale(&img))).to_vec()
            }
            ImageFormat::Rg8_Bc5 => bytemuck::cast_slice((&bc5::encode_color(&img))).to_vec(),
            ImageFormat::Rgb8_Bc7 => bytemuck::cast_slice(&bc7::encode(&img)).to_vec(),
        };
        array.push(bytes);
    }
    Dds {
        magic: header.magic,
        header: header.header,
        dx10_header: header.dx10_header,
        data: array,
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let img = open(&args.file).unwrap();
    let out_file = args.out.unwrap_or_else(|| format!("{}.dds", args.file));

    info!("Encoding {in} with mode {r}{g}{b}{l}{a}{d}{w}{c} to {out_file}", in = args.file,
        r = if args.red { "r" } else { "" },
        g = if args.green { "g" } else { "" },
        b = if args.blue { "b" } else { "" },
        l = if args.luma { "l" } else { "" },
        a = if args.alpha { "a" } else { "" },
        c = if args.compress { "c" } else { "" },
        d = 'u',
        w = '8',);

    if args.compress {
        warn!("Compression is currently unsupported");
    }

    let format = match (
        args.red || args.luma,
        args.green,
        args.blue,
        args.alpha,
        args.compress,
    ) {
        (true, false, false, false, false) => ImageFormat::Luma8,
        (true, false, false, false, true) => ImageFormat::Luma8_Bc4,
        (true, false, false, true, false) => ImageFormat::LumaAlpha8,
        (true, false, false, true, true) => ImageFormat::LumaAlpha8_Bc5,
        (true, true, false, false, true) => ImageFormat::Rg8_Bc5,
        (true, true, true, false, true) => ImageFormat::Rgb8_Bc7,
        (true, true, true, false, false) => ImageFormat::Rgb8,
        (true, true, true, true, _) => ImageFormat::Rgba8,
        _ => ImageFormat::Rgba8,
    };
    let dds = encode_dds(&[img], format);
    // write out file
    let mut file = File::create(out_file).unwrap();
    file.write_all(&dds.magic.to_ne_bytes()).unwrap();
    file.write_all(bytemuck::bytes_of(&dds.header)).unwrap();
    if let Some(dx10_header) = dds.dx10_header {
        file.write_all(bytemuck::bytes_of(&dx10_header)).unwrap();
    }
    for buf in dds.data {
        file.write_all(&buf).unwrap();
    }
}
