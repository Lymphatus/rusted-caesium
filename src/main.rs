extern crate libc;

use std::os::raw::c_char;
use std::ffi::CString;

#[repr(C)]
struct CsJpegPars
{
    quality: i32,
    exif_copy: bool,
    dct_method: i32,
    scale_factor: f32,
}

#[repr(C)]
struct CsPngPars
{
    iterations: i32,
    iterations_large: i32,
    block_split_strategy: i32,
    lossy_8: bool,
    transparent: bool,
    auto_filter_strategy: u32,
    scale_factor: f32,
}

#[repr(C)]
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
    let jpeg_pars = CsJpegPars {
        quality: 80,
        exif_copy: false,
        dct_method: 4,
        scale_factor: 1.0,
    };

    let png_pars = CsPngPars{
        iterations: 10,
        iterations_large: 5,
        block_split_strategy: 4,
        lossy_8: true,
        transparent: true,
        auto_filter_strategy: 0,
        scale_factor: 1.0
    };

    let cs_pars = CsImagePars{
        jpeg: jpeg_pars,
        png: png_pars
    };

    let origin_str = CString::new(b"/Users/lymphatus/Pictures/Tanzania POST/IMG_1278.jpg" as &[u8]).unwrap();
    let origin: *const c_char = origin_str.as_ptr();
    let destination_str = CString::new(b"/Users/lymphatus/Pictures/hydroxide_test/IMG_1278.jpg" as &[u8]).unwrap();
    let destination: *const c_char = destination_str.as_ptr();


    unsafe { cs_compress(origin, destination, cs_pars); }
}
