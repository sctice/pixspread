extern crate argparse;
extern crate raster;

use argparse::{ArgumentParser, Store, StoreTrue};
use raster::{Image, error::RasterError};

#[derive(Clone, Copy)]
enum Offset {
    Row(u32),
    Col(u32),
}

struct Options {
    input_path: String,
    offset: Offset,
    output_path: String,
}

fn main() {
    let options = parse_options();
    std::process::exit(match run(options) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}

fn run(options: Options) -> Result<(), String> {
    let mut image = raster::open(&options.input_path).map_err(|err| match err {
        RasterError::Io(io_err) => format!("{}: {}", options.input_path, io_err),
        _ => format!("{:?}", err)
    })?;

    expand_image_pixels(&mut image, options.offset).map_err(|err| match err {
        RasterError::PixelOutOfBounds(x, y) => format!("Pixel out of bounds: ({}, {})", x, y),
        _ => format!("{:?}", err)
    })?;

    raster::save(&image, &options.output_path).map_err(|err| match err {
        RasterError::Io(io_err) => format!("{}: {}", options.output_path, io_err),
        _ => format!("{:?}", err)
    })
}

fn parse_options() -> Options {
    let mut options = Options {
        input_path: String::from(""),
        offset: Offset::Col(0),
        output_path: String::from(""),
    };

    let mut offset = 0;
    let mut orient_row = false;

    // Limit scope of mutable borrows.
    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Expand a column or row of pixels to the dimensions of the image");

        ap.refer(&mut options.input_path)
            .add_option(&["-i", "--input"], Store, "The image from which to read pixel data")
            .metavar("INPUT_PATH")
            .required();

        ap.refer(&mut offset)
            .add_option(&["-p", "--pixel"], Store, "The column or row offset in pixels")
            .metavar("PIXEL_OFFSET")
            .required();

        ap.refer(&mut orient_row)
            .add_option(&["-r", "--row"], StoreTrue, "Expand a row of pixels instead of a column");

        ap.refer(&mut options.output_path)
            .add_option(&["-o", "--out"], Store, "The path to write to")
            .metavar("OUTPUT_PATH")
            .required();

        ap.parse_args_or_exit();
    }

    options.offset = match orient_row {
        true => Offset::Row(offset),
        false => Offset::Col(offset),
    };

    options
}

fn expand_image_pixels(image: &mut Image, offset: Offset) -> Result<(), RasterError> {
    match offset {
        Offset::Col(x) => expand_image_col(image, x as i32),
        Offset::Row(y) => expand_image_row(image, y as i32),
    }
}

fn expand_image_col(image: &mut Image, x: i32) -> Result<(), RasterError> {
    for write_y in 0..image.height {
        for write_x in 0..image.width {
            let p = image.get_pixel(x, write_y)?;
            image.set_pixel(write_x, write_y, p)?
        }
    }
    Ok(())
}

fn expand_image_row(image: &mut Image, y: i32) -> Result<(), RasterError> {
    for write_y in 0..image.height {
        for write_x in 0..image.width {
            let p = image.get_pixel(write_x, y)?;
            image.set_pixel(write_x, write_y, p)?
        }
    }
    Ok(())
}
