extern crate libc;

use std::os::raw::c_char;
use std::ffi::CString;
use std::env;

#[repr(C)]
#[derive(Debug)]
struct CsJpegPars
{
    quality: i32,
    exif_copy: bool,
    dct_method: i32,
    scale_factor: f64,
}

#[repr(C)]
#[derive(Debug)]
struct CsPngPars
{
    iterations: i32,
    iterations_large: i32,
    block_split_strategy: i32,
    lossy_8: bool,
    transparent: bool,
    auto_filter_strategy: bool,
    scale_factor: f64,
}

#[repr(C)]
#[derive(Debug)]
struct CsImagePars
{
    jpeg: CsJpegPars,
    png: CsPngPars,
}

#[link(name = "caesium")]
extern {
    fn cs_compress(origin: *const c_char, destination: *const c_char, pars: CsImagePars) -> bool;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = args[1].as_bytes();
    let output = args[2].as_bytes();

    let jpeg_pars = CsJpegPars {
        quality: 0,
        exif_copy: false,
        dct_method: 4,
        scale_factor: 1.0,
    };

    let png_pars = CsPngPars{
        iterations: 2,
        iterations_large: 1,
        block_split_strategy: 4,
        lossy_8: true,
        transparent: true,
        auto_filter_strategy: false,
        scale_factor: 1.0
    };

    let cs_pars = CsImagePars{
        jpeg: jpeg_pars,
        png: png_pars
    };

    let origin_str = CString::new(input).unwrap();
    let origin: *const c_char = origin_str.as_ptr();
    let destination_str = CString::new(output).unwrap();
    let destination: *const c_char = destination_str.as_ptr();

    unsafe {
        // let cs_pars: CsImagePars = initialize_parameters();
        cs_compress(origin, destination, cs_pars);
    }
}
