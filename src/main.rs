extern crate libc;

use std::path::PathBuf;
use structopt::StructOpt;

use std::os::raw::c_char;
use std::ffi::CString;

mod cspars;

#[link(name = "caesium")]
extern {
    fn cs_compress(origin: *const c_char, destination: *const c_char, pars: cspars::CsImagePars) -> bool;
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

    let mut cs_pars: cspars::CsImagePars = Default::default();
    cs_pars.jpeg.quality = opt.quality;

    let origin_str = CString::new(input).unwrap();
    let origin: *const c_char = origin_str.as_ptr();
    let destination_str = CString::new(output).unwrap();
    let destination: *const c_char = destination_str.as_ptr();

    unsafe {
        cs_compress(origin, destination, cs_pars);
    }
}
