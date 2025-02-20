use clap::Parser;
use flate2::read::GzDecoder;
use glob::glob;
use notify::{Event, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::{path::Path, sync::mpsc};
use tar::Archive;

// Program that watches for artifact files to watch for and unzips them.
// By default it will watch the current directory for files with the name pattern: artifact_{something}.tar.gz
// When it finds a file with that name pattern it will unzip it and open the index.html file in the unzipped directory.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    // The directory to watch for files to unzip.
    #[arg(short, long, default_value = ".")]
    dir_to_watch: Option<PathBuf>,
    // One shot file to unzip and open.
    #[arg(short = 'o', long)]
    file: Option<PathBuf>,

    // The pattern to watch for that will be unzipped.
    #[arg(short, long, default_value = r"^artifact_\{.*\}\.tar\.gz$")]
    pattern_to_watch_for: Option<String>,
    // The pattern of the file that will be opened.
    #[arg(short, long, default_value = "**/index.html")]
    file_pattern_to_open: Option<String>,
}

fn main() -> notify::Result<()> {
    let args = Args::parse();

    if args.file.is_some() {
        let file = args.file.unwrap();
        unzip_file_and_open(file).unwrap();
        return Ok(());
    }

    let dir = args.dir_to_watch.unwrap();

    watch_dir(dir)?;

    Ok(())
}

fn watch_dir(dir: PathBuf) -> notify::Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    // Only watch the directory given.
    // If we find the file we want to unzip we will just pass the dir to find.
    watcher.watch(Path::new(&dir), RecursiveMode::NonRecursive)?;

    // A number of events can be received for the same file,
    // so we keep track of the last file we unzipped ensuring we only do it once.
    let mut last_unziped = PathBuf::new();

    for res in rx {
        match res {
            Ok(event) => {
                let file = event.paths[0].clone();
                if last_unziped != file && is_tar_gz(file.clone()) {
                    match unzip_file_and_open(file.clone()) {
                        Ok(path) => {
                            last_unziped = file.clone();
                            println!("Unzipped file: {:?}", path);
                        }
                        Err(e) => println!("Error unzipping file: {:?}", e),
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn unzip_file_and_open(file: PathBuf) -> Result<String, String> {
    match unzip_file(file.clone()) {
        Ok(path) => {
            println!("Unzipped file: {:?}", path);
            match find_index_file(path.clone().into()) {
                Ok(path) => {
                    open::that(path.to_str().unwrap()).unwrap();
                    Ok(path.to_str().unwrap().to_string())
                }
                Err(e) => Err(format!("Error finding index file: {:?}", e)),
            }
        }
        Err(e) => Err(format!("Error unzipping file: {:?}", e)),
    }
}

fn is_tar_gz(file: PathBuf) -> bool {
    let regex = Args::parse().pattern_to_watch_for.unwrap();
    let regex = Regex::new(&regex).unwrap();
    regex.is_match(file.file_name().unwrap().to_str().unwrap())
}

fn unzip_file(file: PathBuf) -> Result<String, io::Error> {
    let tar_gz = File::open(file.clone())?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let file_stem = file.file_stem().unwrap().to_string_lossy();
    let name_without_extension = file_stem[..file_stem.len() - 4].to_string() + "_unziped";
    let extract_to = file
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No parent directory"))?
        .join(name_without_extension);
    archive.unpack(extract_to.clone())?;
    println!(
        "File successfully unziped: {:?}\n\n og file: {:?}",
        extract_to, file
    );
    Ok(extract_to.to_str().unwrap().to_string())
}

fn find_index_file(file: PathBuf) -> Result<PathBuf, PathBuf> {
    if !file.is_dir() {
        return Err(file);
    }

    let mut shortest_path: Option<PathBuf> = None;
    let mut shortest_depth = usize::MAX;
    let file_pattern = Args::parse().file_pattern_to_open.unwrap();

    for entry in glob(file.join(file_pattern).to_str().unwrap()).unwrap() {
        match entry {
            Ok(path) => {
                let depth = path.components().count();
                if depth < shortest_depth {
                    shortest_depth = depth;
                    shortest_path = Some(path);
                }
            }
            Err(_) => {
                return Err(file);
            }
        }
    }

    shortest_path.ok_or(file)
}
