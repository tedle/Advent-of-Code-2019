struct ImageLayer {
    pixels: Vec<u8>,
}
struct Image {
    layers: Vec<ImageLayer>,
}

fn parse_image(filename: &str, width: usize, height: usize) -> Image {
    let mut data = String::from(std::fs::read_to_string(filename).unwrap().trim());
    let mut layers: Vec<ImageLayer> = vec![];
    while data.len() > 0 {
        let pixels: Vec<u8> = data
            .drain(..width * height)
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();
        layers.push(ImageLayer { pixels });
    }
    Image { layers }
}

fn decode_image(image: &Image) -> ImageLayer {
    let mut composite_layer = ImageLayer { pixels: vec![] };
    if image.layers.len() == 0 {
        return composite_layer;
    }
    for i in 0..image.layers[0].pixels.len() {
        let mut pixel = 2;
        for layer in &image.layers {
            if layer.pixels[i] != 2 {
                pixel = layer.pixels[i];
                break;
            }
        }
        composite_layer.pixels.push(pixel);
    }
    composite_layer
}

fn find_least(image: &Image, n: u8) -> Option<&ImageLayer> {
    let mut best_layer: Option<&ImageLayer> = None;
    let mut best_layer_count: Option<usize> = None;
    for layer in &image.layers {
        let layer_count = layer.pixels.iter().filter(|p| **p == n).count();
        if best_layer.is_none() || layer_count < best_layer_count.unwrap() {
            best_layer = Some(layer);
            best_layer_count = Some(layer_count);
        }
    }
    best_layer
}

fn main() {
    let image = parse_image("input", 25, 6);
    println!("8-1:");
    let layer = find_least(&image, 0).unwrap();
    let verification_code = layer.pixels.iter().filter(|p| **p == 1).count()
        * layer.pixels.iter().filter(|p| **p == 2).count();
    println!("{}", verification_code);
    println!("8-2:");
    let image_string = decode_image(&image)
        .pixels
        .iter()
        .enumerate()
        .flat_map(|(i, n)| {
            if i % 25 == 0 && i > 0 {
                std::iter::once(format!("\n{}", n.to_string()))
            } else {
                std::iter::once(n.to_string())
            }
        })
        .collect::<String>();
    println!("{}", image_string);
}
