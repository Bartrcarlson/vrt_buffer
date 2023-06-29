//! This program takes a directory of geotiff files and adds a margin to them.
//! The margin is added by using a vrt file as a reference.
//! the program can then crop the buffered files back down to the original size
//! this functionality is useful for raster calculations that use a  search radius
//! to avoid edge effects.
//! # Example
//! ```
//! use std::path::Path;
//! use vrt_buffer::vrt_buffer;
//! use vrt_buffer::crop_down_to_size;
//!
//! let input_dir = Path::new("data");
//! let padded_output_dir = Path::new("output/padded");
//! let trimmed_output_dir = Path::new("output/trimmed");
//! let vrt_file = Path::new("data/data.vrt");
//! let margin = 10;
//!
//! vrt_buffer(&input_dir, &padded_output_dir, &vrt_file, margin).unwrap();
//! // do some calculations with the buffered files
//! crop_down_to_size(&input_dir, &padded_output_dir, &trimmed_output_dir).unwrap();
//!

mod args;
use args::BufferCliArgs;
use clap::Parser;
use vrt_buffer::{crop_down_to_size, vrt_buffer};

fn main() {
    let cli_args = BufferCliArgs::parse();
    match cli_args.subcmd {
        args::Subaction::Pad(pad_args) => {
            vrt_buffer(
                &pad_args.input,
                &pad_args.output,
                &pad_args.vrt,
                pad_args.pad as usize,
            )
            .unwrap();
        }
        args::Subaction::Crop(crop_args) => {
            crop_down_to_size(&crop_args.orginal, &crop_args.input, &crop_args.output).unwrap();
        }
    }
}
