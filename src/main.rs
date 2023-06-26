use gdal::{
    raster::{Buffer, GdalType, RasterBand, ResampleAlg},
    vector::{Field, Geometry, Layer},
    Dataset, Driver, DriverManager, LayerOptions,
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

const INPUT_DIR: &str = "data/";
const OUTPUT_DIR: &str = "output/padded/";
const VRT_FILE: &str = "data/data.vrt";
const MARGIN: usize = 100;

pub fn vrt_buffer() -> Result<(), Box<dyn Error>> {
    // check if output directory exists and create it if not
    fs::create_dir_all(OUTPUT_DIR)?;

    // Load VRT once for efficiency
    let vrt_ds = Dataset::open(&Path::new(VRT_FILE))?;
    let vrt_band = vrt_ds.rasterband(1)?;

    // Get the list of geotiff files in the input directory
    let paths = fs::read_dir(INPUT_DIR)?;

    // For each file in the directory, add margins and save to the output directory
    for path in paths {
        let path = path?.path();
        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extension == "tif" || extension == "tiff" {
                let output_path = Path::new(OUTPUT_DIR).join(path.file_name().unwrap());
                add_margin_to_geotiff(&path, &output_path, MARGIN, &vrt_band, &vrt_ds)?;
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
    let band = ds.rasterband(1)?;

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
    let new_data = vrt_band.read_as::<f64>(
        (xoff, yoff),
        (cols as usize, rows as usize),
        (cols as usize, rows as usize),
        None,
    )?;

    // Create a new geotiff file
    let driver = DriverManager::get_driver_by_name("GTiff")?;
    let mut new_ds = driver.create_with_band_type::<f64, _>(
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

    // create an output raster with the same size as the original raster
    // but the same data as the buffered raster
    let driver = DriverManager::get_driver_by_name("GTiff")?;
    let mut dso_out = driver.create_with_band_type::<f64, _>(
        output_raster.to_str().unwrap(),
        dso.raster_size().0 as isize,
        dso.raster_size().1 as isize,
        1,
    )?;
    dso_out.set_geo_transform(&dso.geo_transform().unwrap())?;
    dso_out.set_projection(&projo)?;

    Ok(())
}
fn main() {
    vrt_buffer().unwrap();
}
