extern crate image;
extern crate simon;

use simon::*;

fn main() {
    let in_path = opt_required::<String>("i", "in", "path to input image file", "PATH")
        .with_help_default()
        .parse_env_default_or_exit();
    let in_image = image::open(in_path).unwrap().to_rgb();
    for y in 0..in_image.height() {
        for x in 0..in_image.width() {
            let [r, g, b] = in_image.get_pixel(x, y).data;
            let ch = match (r, g, b) {
                (0, 0, 0) => '#',
                (255, 255, 255) => '.',
                (0, 0, 255) => '$',
                (255, 0, 0) => '?',
                other => panic!("unrecognised colour: {:?}", other),
            };
            print!("{}", ch);
        }
        println!("");
    }
}
