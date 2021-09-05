extern crate libc;
extern crate num_cpus;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use indicatif::ProgressDrawTarget;
use std::process::exit;
use threadpool::ThreadPool;
use caesium;

mod scanfiles;

#[derive(StructOpt, Debug)]
#[structopt(name = "CaesiumCLT", about = "Caesium Command Line Tools")]
struct Opt {
    /// sets output file quality between [0-100], 0 for optimization
    #[structopt(short = "q", long)]
    quality: u32,

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

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let args: Vec<PathBuf> = opt.files;

    if args.len() == 0 {
        println!("Please provide at least one file.");
        exit(-1);
    }

    let folder_only = args.len() == 1 && args[0].is_dir();
    if !folder_only && opt.keep_structure {
        println!("Option -S (--keep-structure) can be used only if the input is a folder.");
        exit(-1);
    }
    let input_dir = args[0].clone().into_os_string();

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    let output_dir = opt.output.clone();
    std::fs::create_dir_all(output_dir).unwrap();

    let files: Vec<PathBuf> = scanfiles::scanfiles(args, opt.recursive);

    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_draw_target(ProgressDrawTarget::stdout_nohz());
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}\n{msg}")
        .progress_chars("#>-"));
    let pb_arc = Arc::new(progress_bar);

    let errors: Vec<(String, String)> = vec![];
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
                Ok(relative_path) => { input_filename = relative_path.to_path_buf().into_os_string() }
                Err(e) => {
                    println!("{} - {:?}", e, out_relative_path);
                    exit(-1);
                }
            }
            let mut base_folder = opt.output.clone();
            base_folder.push(input_filename.clone());
            base_folder.pop();
            std::fs::create_dir_all(base_folder).unwrap();
        }
        output_buf.push(input_filename);
        let output = output_buf.into_os_string().into_string().unwrap();

        let mut r_parameters = caesium::initialize_parameters();
        r_parameters.jpeg.quality = opt.quality;
        r_parameters.png.level = get_png_level(opt.quality);
        r_parameters.keep_metadata = opt.exif;

        let errors_arc_clone = Arc::clone(&errors_arc);

        pool.execute(move || {
            pb.set_message(format!("{:?}", input.clone()));
            let result = caesium::compress(input, output, r_parameters);

            match result {
                Ok(r) => r,
                Err(e) => {
                    let mut v = errors_arc_clone.lock().unwrap();
                    v.push((file_path_clone.to_string(), e.to_string()));
                }
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


fn get_png_level(quality: u32) -> u32 {
    match quality {
        1..=39 => 0,
        40..=49 => 1,
        50..=59 => 2,
        60..=69 => 3,
        70..=79 => 5,
        80..=89 => 6,
        _ => 0
    }
}
