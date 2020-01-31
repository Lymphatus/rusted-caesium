extern crate libc;

use libc::c_int;

#[repr(C)]
pub struct CsJpegPars {
    pub quality: c_int,
    pub exif_copy: bool,
    dct_method: i32,
    pub scale_factor: f64,
}

#[repr(C)]
pub struct CsPngPars {
    iterations: i32,
    iterations_large: i32,
    block_split_strategy: i32,
    lossy_8: bool,
    transparent: bool,
    auto_filter_strategy: bool,
    pub scale_factor: f64,
}

#[repr(C)]
pub struct CsImagePars {
    pub jpeg: CsJpegPars,
    pub png: CsPngPars,
}

impl Default for CsImagePars {
    fn default() -> CsImagePars {
        CsImagePars {
            jpeg: Default::default(),
            png: Default::default(),
        }
    }
}

impl Default for CsJpegPars {
    fn default() -> CsJpegPars {
        CsJpegPars {
            quality: 0,
            exif_copy: false,
            dct_method: 2048,
            scale_factor: 1.0,
        }
    }
}

impl Default for CsPngPars {
    fn default() -> CsPngPars {
        CsPngPars {
            iterations: 2,
            iterations_large: 1,
            block_split_strategy: 2,
            lossy_8: true,
            transparent: true,
            auto_filter_strategy: false,
            scale_factor: 1.0,
        }
    }
}
