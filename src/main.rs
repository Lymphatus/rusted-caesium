extern crate libc;
extern crate num_cpus;

use libc::c_int;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

use std::ffi::CString;
use std::os::raw::c_char;

use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use indicatif::ProgressDrawTarget;

use std::process::exit;
use threadpool::ThreadPool;

mod cspars;
mod scanfiles;

#[link(name = "caesium")]
extern "C" {
    fn cs_compress(
        origin: *const c_char,
        destination: *const c_char,
        pars: &cspars::CsImagePars,
        return_value: &i32
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
    let folder_only = args.len() == 1 && args[0].is_dir();
    if !folder_only && opt.keep_structure {
        println!("Option -S (--keep-structure) can be used only if the input is a folder.");
        exit(-1);
    }
    let input_dir = args[0].clone().into_os_string();

    let mut cs_pars: cspars::CsImagePars = Default::default();
    cs_pars.jpeg.quality = opt.quality as c_int;
    cs_pars.jpeg.exif_copy = opt.exif;
    cs_pars.jpeg.scale_factor = opt.scale;
    cs_pars.png.scale_factor = opt.scale;

    let pars_arc = Arc::new(cs_pars);

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    let files: Vec<PathBuf> = scanfiles::scanfiles(args, opt.recursive);

    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_draw_target(ProgressDrawTarget::stdout_nohz());
    progress_bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}\n{msg}")
            .progress_chars("#>-"));
    let pb_arc = Arc::new(progress_bar);

    let errors: Vec<(String, i32)> = vec![];
    let errors_arc = Arc::new(Mutex::new(errors));

    for input_file in files.into_iter() {
        let pb = pb_arc.clone();
        let mut input_filename = input_file.file_name().unwrap().to_os_string();
        let input = input_file.clone().into_os_string().into_string().unwrap();
        let file_path_clone = Arc::clone(&Arc::new(input.clone()));

        let mut output_buf: PathBuf = opt.output.clone();
        if opt.keep_structure {
            let out_relative_path = input_file.clone();
            match out_relative_path.strip_prefix(input_dir.clone()) {
                Ok(relative_path) => {input_filename = relative_path.to_path_buf().into_os_string()},
                Err(e) => {println!("{} - {:?}", e, out_relative_path); exit(-1);}
            }
            let mut base_folder = opt.output.clone();
            base_folder.push(input_filename.clone());
            base_folder.pop();
            std::fs::create_dir_all(base_folder).unwrap();
        }
        output_buf.push(input_filename);
        let output = output_buf.into_os_string().into_string().unwrap();
        let passed_pars = pars_arc.clone();
        let destination_str = CString::new(output).unwrap();

        let origin_str = CString::new(input).unwrap();

        let errors_arc_clone = Arc::clone(&errors_arc);
        
        pool.execute(move || {
            let origin: *const c_char = origin_str.as_ptr();
            let destination: *const c_char = destination_str.as_ptr();
            pb.set_message(&format!("{:?}", destination_str));
            let compress_return_value = 0;
            unsafe {
                cs_compress(origin, destination, &passed_pars, &compress_return_value);
            }
            if compress_return_value != 0 {
                let mut v = errors_arc_clone.lock().unwrap();
                v.push((file_path_clone.to_string(), compress_return_value));
            }
            pb.inc(1);
        });
        
    }

    pool.join();
    pb_arc.finish_with_message("done");
    for errors in Arc::try_unwrap(errors_arc).unwrap().into_inner().unwrap().into_iter() {
        println!("{} failed with code {}", errors.0, errors.1);
    }

    exit(0);
}
