extern crate libc;

use std::path::PathBuf;
use structopt::StructOpt;

use std::os::raw::c_char;
use std::ffi::CString;

#[repr(C)]
struct CsJpegPars
{
    quality: i32,
    exif_copy: bool,
    dct_method: i32,
    scale_factor: f64,
}

#[repr(C)]
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
struct CsImagePars
{
    jpeg: CsJpegPars,
    png: CsPngPars,
}

#[link(name = "caesium")]
extern {
    fn cs_compress(origin: *const c_char, destination: *const c_char, pars: CsImagePars) -> bool;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "CaesiumCLT", about = "Caesium Command Line Tools")]
struct Opt {
    /// sets output file quality between [0-100], 0 for optimization
    #[structopt(short = "q", long)]
    quality: i32,

    /// keeps EXIF info during compression
    #[structopt(short = "e", long)]
    exif: bool,

    /// output folder
    #[structopt(short = "o", long, parse(from_os_str))]
    output: PathBuf,

    /// scale the image, using a floating point scale factor (eg. 0.5)
    #[structopt(short = "s", long, default_value = "1")]
    scale: f64,

    /// if input is a folder, scan subfolders too
    #[structopt(short = "R", long)]
    recursive: bool,

    /// keep the folder structure, use with -R
    #[structopt(short = "S", long)]
    keep_structure: bool,

    /// do not really compress files but just show output paths
    #[structopt(short = "d", long)]
    dry_run: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let mut args: Vec<PathBuf> = opt.files;

    let input = args.remove(0).into_os_string().into_string().unwrap();
    let output = opt.output.into_os_string().into_string().unwrap();

    let jpeg_pars = CsJpegPars {
        quality: opt.quality,
        exif_copy: false,
        dct_method: 4,
        scale_factor: 1.0,
    };

    let png_pars = CsPngPars {
        iterations: 2,
        iterations_large: 1,
        block_split_strategy: 4,
        lossy_8: true,
        transparent: true,
        auto_filter_strategy: false,
        scale_factor: 1.0,
    };

    let cs_pars = CsImagePars {
        jpeg: jpeg_pars,
        png: png_pars,
    };

    let origin_str = CString::new(input).unwrap();
    let origin: *const c_char = origin_str.as_ptr();
    let destination_str = CString::new(output).unwrap();
    let destination: *const c_char = destination_str.as_ptr();

    unsafe {
        cs_compress(origin, destination, cs_pars);
    }
}
