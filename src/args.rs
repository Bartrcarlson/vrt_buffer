use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, clap::Parser)]
#[clap(author = "Bart Carlson", version = "1.0")]
pub struct BufferCliArgs {
    #[clap(subcommand)]
    pub subcmd: Subaction,
}
#[derive(Debug, Subcommand)]
pub enum Subaction {
    /// pads the raster file with a border of additional pixels who are sourced from the adjacent
    /// rasters using a vrt file
    Pad(PadCommand),

    /// crops the processed raster to the extent of the original raster
    Crop(CropCommand),
}

#[derive(Debug, Args)]
pub struct PadCommand {
    /// the input raster directory
    #[clap(short, long)]
    pub input: PathBuf,

    /// the output raster directory
    #[clap(short, long)]
    pub output: PathBuf,

    /// the vrt file that describes the subject area including the adjacent rasters
    #[clap(short, long)]
    pub vrt: PathBuf,

    /// the number of pixels to pad the raster with
    #[clap(short, long)]
    pub pad: u32,
}

#[derive(Debug, Args)]
pub struct CropCommand {
    /// the original raster directory used for knowing the extent to crop to
    #[clap(short = 'g', long = "original")]
    pub original: PathBuf,

    /// the input raster directory
    #[clap(short = 'i', long = "input")]
    pub input: PathBuf,

    /// the output raster directory
    #[clap(short = 'o', long = "output")]
    pub output: PathBuf,
}
