//! # vrt_buffer
//!
//! `vrt_buffer` is a crate that provides functions for adding a margin to geotiff files using a VRT file as a reference,
//! as well as cropping the buffered files back to the original size.
//!
//! ## Example
//!
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
//! ```
//!
//! ## Usage
//! ### rust api
//! The crate provides two main functions:
//!
//! - `vrt_buffer`: Adds a margin to geotiff files using a VRT file as a reference.
//! - `crop_down_to_size`: Crops the buffered files back to the original size.
//!
//! Refer to the individual function documentation for more details on their usage.
//!
//! ### Command line
//! the clap framwork is also used to provide a command line interface for the crate.
//! the easiest way to use the CLI is to run
//!```zsh
//! cargo run --release -- -h
//!```
//! to get a list of the available commands.
//!
//! ## Installation
//! gdal must be installed and the path environment variable must be set to the gdal binaries.
//! build using
//!```zsh
//! cargo build --release
//! cargo install --path .
//! ```
//! uninstall with
//! ```zsh
//! cargo uninstall vrt_buffer
//! ```
//!
use gdal::{raster::RasterBand, Dataset, DriverManager};
use std::{error::Error, fs, path::Path};

/// adds a margin to the geotiff files in the input directory and saves them to the output directory.
/// The margin is added by using the vrt file as a reference.
/// input_dir: directory of the original files
/// output_dir: directory to save the buffered files
/// vrt_file: vrt file of the original files
/// margin: size of the margin to add to the files
pub fn vrt_buffer(
    input_dir: &Path,
    output_dir: &Path,
    vrt_file: &Path,
    margin: usize,
) -> Result<(), Box<dyn Error>> {
    // check if output directory exists and create it if not
    fs::create_dir_all(output_dir)?;

    // Load VRT once for efficiency
    let vrt_ds = Dataset::open(vrt_file)?;
    let vrt_band = vrt_ds.rasterband(1)?;

    // Get the list of geotiff files in the input directory
    let paths = fs::read_dir(input_dir)?;

    // For each file in the directory, add margins and save to the output directory

    for path in paths {
        let path = match path {
            Ok(path) => path.path(),
            Err(_) => {
                eprintln!("Error processing path. Skipping...");
                continue;
            }
        };
        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extension == "tif" || extension == "tiff" {
                let output_file_name = match path.file_name() {
                    Some(file_name) => file_name,
                    None => {
                        eprintln!(
                            "Could not compose a output file name based on {:?}. Skipping...",
                            path
                        );
                        continue;
                    }
                };
                let output_path = Path::new(output_dir).join(output_file_name);
                match add_margin_to_geotiff(&path, &output_path, margin, &vrt_band, &vrt_ds) {
                    Ok(_) => (),
                    Err(_) => eprintln!("Error adding margin to geotiff. Skipping..."),
                }
            }
        }
    }

    Ok(())
}
/// takes a directory of the original directory with the tif files that where buffered and
/// uses them as the reference to trim the buffered files to the original size
/// org_dir: directory of the original files
/// input_dir: directory of the buffered files
/// output_dir: directory to save the trimmed files
pub fn crop_down_to_size(
    org_dir: &Path,
    input_dir: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output_dir)?;
    let paths = fs::read_dir(input_dir)?;

    for path in paths {
        let path = match path {
            Ok(path) => path.path(),
            Err(_) => {
                eprintln!("Error processing path. Skipping...");
                continue;
            }
        };
        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extension == "tif" || extension == "tiff" {
                let file_name = match path.file_name() {
                    Some(file_name) => file_name,
                    None => {
                        eprintln!("Could not retrieve file name from {:?}. Skipping...", path);
                        continue;
                    }
                };
                let input_path = org_dir.join(file_name);
                let output_path = output_dir.join(file_name);
                match trim_buffered_to_size(&input_path, &path, &output_path) {
                    Ok(_) => (),
                    Err(_) => eprintln!("Error trimming buffered size. Skipping..."),
                }
            }
        }
    }

    Ok(())
}

fn add_margin_to_geotiff(
    file_path: &Path,
    output_path: &Path,
    margin: usize,
    vrt_band: &RasterBand,
    vrt_ds: &Dataset,
) -> Result<(), Box<dyn Error>> {
    // Open the geotiff file
    let ds = match Dataset::open(file_path) {
        Ok(ds) => ds,
        Err(e) => return Err(Box::new(e)),
    };

    // Get the original geotiff's data and metadata
    let geotransform = match ds.geo_transform() {
        Ok(geotransform) => geotransform,
        Err(e) => return Err(Box::new(e)),
    };
    let projection = ds.projection();

    // Compute expanded geotransform
    let mut new_geotransform = geotransform;
    new_geotransform[0] -= (margin as f64) * geotransform[1]; // x_origin
    new_geotransform[3] -= (margin as f64) * geotransform[5]; // y_origin

    // Read data from the VRT
    let vrt_geotransform = match vrt_ds.geo_transform() {
        Ok(vrt_geotransform) => vrt_geotransform,
        Err(e) => return Err(Box::new(e)),
    };
    let xoff = ((new_geotransform[0] - vrt_geotransform[0]) / vrt_geotransform[1])
        .max(0.0)
        .floor() as isize;
    let yoff = ((vrt_geotransform[3] - new_geotransform[3]) / vrt_geotransform[5].abs())
        .max(0.0)
        .floor() as isize;

    // Make sure we don't exceed the raster dimensions
    let cols =
        (vrt_ds.raster_size().0 as isize - xoff).min((ds.raster_size().0 + 2 * margin) as isize);
    let rows =
        (vrt_ds.raster_size().1 as isize - yoff).min((ds.raster_size().1 + 2 * margin) as isize);

    let new_data = match vrt_band.read_as::<f32>(
        (xoff, yoff),
        (cols as usize, rows as usize),
        (cols as usize, rows as usize),
        None,
    ) {
        Ok(new_data) => new_data,
        Err(e) => return Err(Box::new(e)),
    };

    // Create a new geotiff file
    let driver = match DriverManager::get_driver_by_name("GTiff") {
        Ok(driver) => driver,
        Err(e) => return Err(Box::new(e)),
    };

    let mut new_ds = match driver.create_with_band_type::<f32, _>(
        output_path.to_str().unwrap(),
        cols as isize,
        rows as isize,
        1,
    ) {
        Ok(new_ds) => new_ds,
        Err(e) => return Err(Box::new(e)),
    };

    if let Err(e) = new_ds.set_geo_transform(&new_geotransform) {
        return Err(Box::new(e));
    };

    if let Err(e) = new_ds.set_projection(&projection) {
        return Err(Box::new(e));
    };

    let mut new_band = match new_ds.rasterband(1) {
        Ok(new_band) => new_band,
        Err(e) => return Err(Box::new(e)),
    };

    if let Err(e) = new_band.write((0, 0), (cols as usize, rows as usize), &new_data) {
        return Err(Box::new(e));
    };

    Ok(())
}
fn trim_buffered_to_size(
    org_raster: &Path,
    buffered_raster: &Path,
    output_raster: &Path,
) -> Result<(), Box<dyn Error>> {
    let dso = match Dataset::open(org_raster) {
        Ok(dso) => dso,
        Err(e) => return Err(Box::new(e)),
    };

    let dsb = match Dataset::open(buffered_raster) {
        Ok(dsb) => dsb,
        Err(e) => return Err(Box::new(e)),
    };

    let projo = dso.projection();

    let geo_transform_o = match dso.geo_transform() {
        Ok(geo_transform) => geo_transform,
        Err(e) => return Err(Box::new(e)),
    };

    let geo_transform_b = match dsb.geo_transform() {
        Ok(geo_transform) => geo_transform,
        Err(e) => return Err(Box::new(e)),
    };

    let x_offset = ((geo_transform_o[0] - geo_transform_b[0]) / geo_transform_b[1]) as usize;
    let y_offset = ((geo_transform_o[3] - geo_transform_b[3]) / geo_transform_b[5]) as usize;

    let band = match dsb.rasterband(1) {
        Ok(band) => band,
        Err(e) => return Err(Box::new(e)),
    };

    let buffered_data = match band.read_as::<f32>(
        (x_offset as isize, y_offset as isize),
        (dso.raster_size().0, dso.raster_size().1),
        (dso.raster_size().0, dso.raster_size().1),
        None,
    ) {
        Ok(buffered_data) => buffered_data,
        Err(e) => return Err(Box::new(e)),
    };

    let driver = match DriverManager::get_driver_by_name("GTiff") {
        Ok(driver) => driver,
        Err(e) => return Err(Box::new(e)),
    };

    let mut dso_out = match driver.create_with_band_type::<f32, _>(
        output_raster.to_str().unwrap(),
        dso.raster_size().0 as isize,
        dso.raster_size().1 as isize,
        1,
    ) {
        Ok(dso_out) => dso_out,
        Err(e) => return Err(Box::new(e)),
    };

    if let Err(e) = dso_out.set_geo_transform(&geo_transform_o) {
        return Err(Box::new(e));
    };

    if let Err(e) = dso_out.set_projection(&projo) {
        return Err(Box::new(e));
    };

    let mut band_out = match dso_out.rasterband(1) {
        Ok(band_out) => band_out,
        Err(e) => return Err(Box::new(e)),
    };

    if let Err(e) = band_out.write(
        (0, 0),
        (dso.raster_size().0, dso.raster_size().1),
        &buffered_data,
    ) {
        return Err(Box::new(e));
    };

    Ok(())
}
