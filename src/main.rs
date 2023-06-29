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
