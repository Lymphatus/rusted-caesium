extern crate libc;
extern crate num_cpus;

use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

use std::ffi::CString;
use std::os::raw::c_char;

// use indicatif::ProgressBar;
use std::process::exit;
// use std::thread;
use threadpool::ThreadPool;
use walkdir::WalkDir;

mod cspars;

#[link(name = "caesium")]
extern "C" {
    fn cs_compress(
        origin: *const c_char,
        destination: *const c_char,
        pars: &cspars::CsImagePars,
    ) -> bool;
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
    let args: Vec<PathBuf> = opt.files;

    let mut cs_pars: cspars::CsImagePars = Default::default();
    cs_pars.jpeg.quality = opt.quality;
    cs_pars.jpeg.exif_copy = opt.exif;
    cs_pars.jpeg.scale_factor = opt.scale;
    cs_pars.png.scale_factor = opt.scale;

    // let pb = ProgressBar::new(args.len() as u64);

    let pars_arc = Arc::new(cs_pars);

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    let mut files: Vec<PathBuf> = vec![];

    for input in args.into_iter() {
        if input.is_dir() {
            for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
                files.push(PathBuf::from(entry.path()));
            }
        } else {
            files.push(input);
        }
    }

    for input_file in files.into_iter() {
        let input_filename = input_file.file_name().unwrap().to_os_string();
        let input = input_file.clone().into_os_string().into_string().unwrap();
        let mut output_buf: PathBuf = opt.output.clone();
        output_buf.push(input_filename);
        let output = output_buf.into_os_string().into_string().unwrap();
        let passed_pars = pars_arc.clone();
        let destination_str = CString::new(output).unwrap();
        let origin_str = CString::new(input).unwrap();

        pool.execute(move || {
            let origin: *const c_char = origin_str.as_ptr();
            let destination: *const c_char = destination_str.as_ptr();
            unsafe {
                cs_compress(origin, destination, &passed_pars);
            }
            println!(
                "{:?} -> {:?}",
                origin_str,
                destination_str,
            );
            // pb.inc(1);
        });
    }

    pool.join();
    // let mut children = vec![];

    // for child in children {
    //     let _ = child.join();
    //     // pb.inc(1);
    // }
    // pb.finish_with_message("done");

    exit(0);
}
