use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::path::PathBuf;
use walkdir::DirEntry;
use walkdir::WalkDir;

use mime_guess;
use mime_guess::mime;

fn is_valid(entry: &DirEntry) -> bool {
    let mut result = true;
    if entry.file_type().is_file() {
        let guess = mime_guess::from_path(entry.path());
        result = match guess.first() {
            Some(t) => t == mime::IMAGE_JPEG || t == mime::IMAGE_PNG,
            None => false
        };
    }

    result
}

pub fn scanfiles(args: Vec<PathBuf>, recursive: bool) -> Vec<PathBuf>{
    let mut files: Vec<PathBuf> = vec![];

    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_message("Collecting files...");
    progress_bar.enable_steady_tick(80);
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}"),
    );
    for input in args.into_iter() {
        if input.exists() && input.is_dir() {
            let mut walk_dir = WalkDir::new(input);
            if !recursive {
                walk_dir = walk_dir.max_depth(1);
            }
            for entry in walk_dir.into_iter().filter_entry(|e| is_valid(e)).filter_map(|e| e.ok()) {
                let entry: PathBuf = PathBuf::from(entry.path());
                if !entry.is_dir() {
                    files.push(entry);
                    progress_bar.tick();
                }
            }
        } else if input.exists() {
            files.push(input);
            progress_bar.tick();
        }
    }

    progress_bar.finish_and_clear();

    files
}