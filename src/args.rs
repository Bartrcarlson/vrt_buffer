use clap::{Args, Parser, Subcommand};

#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
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
    pub input: String,

    /// the output raster directory
    #[clap(short, long)]
    pub output: String,

    /// the vrt file that describes the subject area including the adjacent rasters
    #[clap(short, long)]
    pub vrt: String,

    /// the number of pixels to pad the raster with
    #[clap(short, long)]
    pub pad: u32,
}

#[derive(Debug, Args)]
pub struct CropCommand {
    /// the original raster directory used for knowing the extent to crop to
    pub orginal: String,

    /// the input raster directory
    pub input: String,

    /// the output raster directory
    pub output: String,
}
