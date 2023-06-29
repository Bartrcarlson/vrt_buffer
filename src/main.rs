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
//! let input_dir = Path::new("data");
//! let padded_output_dir = Path::new("output/padded");
//! let trimmed_output_dir = Path::new("output/trimmed");
//! let vrt_file = Path::new("data/data.vrt");
//! let margin = 10;
//! vrt_buffer(&input_dir, &padded_output_dir, &vrt_file, margin).unwrap();
//! // do some calculations with the buffered files
//! crop_down_to_size(&input_dir, &padded_output_dir, &trimmed_output_dir).unwrap();
use gdal::{raster::RasterBand, Dataset, DriverManager};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

/// addeds a margin to the geotiff files in the input directory and saves them to the output directory.
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
        let path = path?.path();
        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extension == "tif" || extension == "tiff" {
                let output_path = Path::new(output_dir).join(path.file_name().unwrap());
                add_margin_to_geotiff(&path, &output_path, margin, &vrt_band, &vrt_ds)?;
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
    fs::create_dir_all(&output_dir)?;
    let paths = fs::read_dir(input_dir)?;
    for path in paths {
        let path = path?.path();
        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extension == "tif" || extension == "tiff" {
                let input_path = org_dir.join(path.file_name().unwrap());
                let output_path = output_dir.join(path.file_name().unwrap());
                trim_buffered_to_size(&input_path, &path, &output_path)?;
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
    let ds = Dataset::open(file_path)?;
    // let band = ds.rasterband(1)?;

    // Get the original geotiff's data and metadata
    let geotransform = ds.geo_transform()?;
    let projection = ds.projection();

    // Compute expanded geotransform
    let mut new_geotransform = geotransform.clone();
    new_geotransform[0] -= (margin as f64) * geotransform[1]; // x_origin
    new_geotransform[3] -= (margin as f64) * geotransform[5]; // y_origin

    // Read data from the VRT
    let xoff = ((new_geotransform[0] - vrt_ds.geo_transform()?[0]) / vrt_ds.geo_transform()?[1])
        .max(0.0)
        .floor() as isize;
    let yoff = ((vrt_ds.geo_transform()?[3] - new_geotransform[3])
        / vrt_ds.geo_transform()?[5].abs())
    .max(0.0)
    .floor() as isize;

    // Make sure we don't exceed the raster dimensions
    let cols =
        (vrt_ds.raster_size().0 as isize - xoff).min((ds.raster_size().0 + 2 * margin) as isize);
    let rows =
        (vrt_ds.raster_size().1 as isize - yoff).min((ds.raster_size().1 + 2 * margin) as isize);
    let new_data = vrt_band.read_as::<f32>(
        (xoff, yoff),
        (cols as usize, rows as usize),
        (cols as usize, rows as usize),
        None,
    )?;

    // Create a new geotiff file
    let driver = DriverManager::get_driver_by_name("GTiff")?;
    let mut new_ds = driver.create_with_band_type::<f32, _>(
        output_path.to_str().unwrap(),
        cols as isize,
        rows as isize,
        1,
    )?;
    new_ds.set_geo_transform(&new_geotransform)?;
    new_ds.set_projection(&projection)?;

    let mut new_band = new_ds.rasterband(1)?;
    new_band.write((0, 0), (cols as usize, rows as usize), &new_data)?;

    Ok(())
}
fn trim_buffered_to_size(
    org_raster: &PathBuf,
    buffered_raster: &PathBuf,
    output_raster: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let dso = Dataset::open(org_raster)?;
    let dsb = Dataset::open(buffered_raster)?;
    let projo = dso.projection();

    // Compute offsets in buffered raster based on geo_transform
    let geo_transform_o = dso.geo_transform()?;
    let geo_transform_b = dsb.geo_transform()?;
    let x_offset = ((geo_transform_o[0] - geo_transform_b[0]) / geo_transform_b[1]) as usize;
    let y_offset = ((geo_transform_o[3] - geo_transform_b[3]) / geo_transform_b[5]) as usize;

    // read data from the buffered raster
    let band = dsb.rasterband(1)?;
    let buffered_data = band.read_as::<f32>(
        (x_offset as isize, y_offset as isize),
        (dso.raster_size().0, dso.raster_size().1),
        (dso.raster_size().0, dso.raster_size().1),
        None,
    )?;

    // create an output raster with the same size as the original raster
    let driver = DriverManager::get_driver_by_name("GTiff")?;
    let mut dso_out = driver.create_with_band_type::<f32, _>(
        output_raster.to_str().unwrap(),
        dso.raster_size().0 as isize,
        dso.raster_size().1 as isize,
        1,
    )?;
    dso_out.set_geo_transform(&geo_transform_o)?;
    dso_out.set_projection(&projo)?;

    // write data to the output raster
    let mut band_out = dso_out.rasterband(1)?;
    band_out.write(
        (0, 0),
        (dso.raster_size().0, dso.raster_size().1),
        &buffered_data,
    )?;

    Ok(())
}
fn main() {
    let org_data = PathBuf::from("data/");
    let padded_data = PathBuf::from("output/data_padded/");
    let trimmed_data = PathBuf::from("output/data_trimmed/");
    let vrt_path = PathBuf::from("data/data.vrt");
    let margin = 100;

    vrt_buffer(&org_data, &padded_data, &vrt_path, margin).unwrap();
    crop_down_to_size(&org_data, &padded_data, &trimmed_data).unwrap();
}
