use hsl::HSL;
use linreg::linear_regression;

/// Computes the average error of mapping a block of values to a given palette
///
/// * `values`: The original dataset
/// * `pal`: Endpoints of the palette
/// * `colors`: Number of colors in the palette
pub fn avg_error(values: &[u8], pal: (u8, u8), colors: usize) -> u8 {
    let mut error = 0;
    for value in values {
        let (i, c) = map_color(*value, pal, colors);
        error += value.abs_diff(c);
    }
    error / values.len() as u8
}

/// Map a color value to it's closest match in a palette
///
/// * `color`: The color we are trying to map
/// * `pal`: The  endpoints of the palette
/// * `colors`: The number of colors in the palette
///
/// # Returns
///  A tuple representing `(index, value)`
pub fn map_color(color: u8, pal: (u8, u8), colors: usize) -> (usize, u8) {
    let min = pal.0.min(pal.1);
    let max = pal.0.max(pal.1);

    let mut closest = (0, min);
    let step = (max - min) as usize / colors;
    for idx in 0..colors {
        let c = min + (step * idx) as u8;
        if color.abs_diff(c) < color.abs_diff(closest.1) {
            closest = (idx, c);
        }
    }
    closest
}

/// Generates endpoints for a set of values
///
/// * `values`: List of values that need to be represented
/// * `colors`: Number of color clusters to base the endpoints on
pub fn generate_palette<T: PartialOrd + PartialEq + Into<f32> + Clone>(values: &[T]) -> (f32, f32) {
    let mut values = values.to_vec();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values.dedup();

    // special case if n = 1;
    if values.len() == 1 {
        return (values[0].clone().into(), values[0].clone().into());
    }
    /*if values.len() == 2 {
        return (values[0].clone().into(), values[1].clone().into());
    }*/

    let values: Vec<f32> = values.iter().map(|v| v.clone().into()).collect();

    let mut indices = vec![0.0; values.len()];
    for (i, e) in indices.iter_mut().enumerate() {
        *e = i as f32;
    }

    let (m, b): (f32, f32) = linear_regression(&indices, &values).unwrap();
    let line = |x| m * x as f32 + b;
    let min = line(0);
    let max = line(values.len() - 1);
    (min, max)
}

/// Thoughts on being able to generate a line for colors:
/// Ok, so when generating colors it may be smart to drop into a HSL like colorspace
/// We want to figure out the lightness range, this is going to be very apparent to the user so.
/// We also don't want to randomize the hue. So we need to figure out the best 2 hues to use.
/// Then we should be able to create our endpoint colors
pub use generate_palette3d_rgb as generate_palette3d;

pub fn generate_palette3d_rgb(values: &[u8]) -> ([u8; 3], [u8; 3]) {
    let values: Vec<[u8; 3]> = values
        .chunks(3)
        .map(|rgb| [rgb[0], rgb[1], rgb[2]])
        .collect();
    let red: Vec<u8> = values.iter().map(|c| c[0]).collect();
    let grn: Vec<u8> = values.iter().map(|c| c[1]).collect();
    let blu: Vec<u8> = values.iter().map(|c| c[2]).collect();

    let red = generate_palette(&red);
    let grn = generate_palette(&grn);
    let blu = generate_palette(&blu);

    let min = [red.0 as u8, grn.0 as u8, blu.0 as u8];
    let max = [red.1 as u8, grn.1 as u8, blu.1 as u8];
    (min, max)
}

pub fn generate_palette3d_hsl(values: &[u8]) -> ([u8; 3], [u8; 3]) {
    let values: Vec<[f32; 3]> = values
        .chunks(3)
        .map(|c| HSL::from_rgb(c))
        .map(|hsl| [hsl.h as f32, hsl.s as f32, hsl.l as f32])
        .collect();
    let hue: Vec<f32> = values.iter().map(|c| c[0]).collect();
    let sat: Vec<f32> = values.iter().map(|c| c[1]).collect();
    let lum: Vec<f32> = values.iter().map(|c| c[2]).collect();

    let hue = generate_palette(&hue);
    let sat = generate_palette(&sat);
    let lum = generate_palette(&lum);

    let min = HSL {
        h: hue.0 as f64,
        s: sat.0 as f64,
        l: lum.0 as f64,
    }
    .to_rgb();

    let max = HSL {
        h: hue.1 as f64,
        s: sat.1 as f64,
        l: lum.1 as f64,
    }
    .to_rgb();
    ([min.0, min.1, min.2], [max.0, max.1, max.2])
}
