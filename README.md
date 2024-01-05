# vrt_buffer

`vrt_buffer` is a crate that provides functions for adding a margin to geotiff files using a VRT file as a reference,
as well as cropping the buffered files back to the original size. this is useful preprocessing step when working with
large datasets of raster files where pixel values at the edges of the files are used in calculations. Computations that
occur using a search radius around each pixel will be incorrect at the edges of the files. by adding a margin to the files
before performing the calculations, the edge pixels will be correct. after the calculations are complete, the files can 
be cropped back to their original size.

## Example

```rust
use std::path::Path;
use vrt_buffer::vrt_buffer;
use vrt_buffer::crop_down_to_size;

let input_dir = Path::new("data");
let padded_output_dir = Path::new("output/padded");
let trimmed_output_dir = Path::new("output/trimmed");
let vrt_file = Path::new("data/data.vrt");
let margin = 10;

vrt_buffer(&input_dir, &padded_output_dir, &vrt_file, margin).unwrap();
// do some calculations with the buffered files
crop_down_to_size(&input_dir, &padded_output_dir, &trimmed_output_dir).unwrap();
```

## Usage
### rust api
The crate provides two main functions:

- `vrt_buffer`: Adds a margin to geotiff files using a VRT file as a reference.
- `crop_down_to_size`: Crops the buffered files back to the original size.

Refer to the individual function documentation for more details on their usage.

### Command line
the clap framwork is also used to provide a command line interface for the crate.
To get a list of the available commands.

```sh
vrt_buffer -h
```

## Installation
gdal must be installed and the path environment variable must be set to the gdal binaries.
```sh
cargo install --git https://github.com/Bartrcarlson/vrt_buffer.git
cargo uninstall vrt_buffer
```
## License
This project is licensed under the MIT License

## Contributing
Pull requests are welcome.
