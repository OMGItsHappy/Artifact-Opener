use flate2::read::GzDecoder;
use notify::{Event, RecursiveMode, Watcher};
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::{path::Path, sync::mpsc};
use tar::Archive;

fn main() -> notify::Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    // Use recommended_watcher() to automatically select the best implementation
    // for your platform. The `EventHandler` passed to this constructor can be a
    // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
    // another type the trait is implemented for.
    let mut watcher = notify::recommended_watcher(tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(
        Path::new("C:\\Users\\oschwab\\Downloads"),
        RecursiveMode::NonRecursive,
    )?;
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => {
                let file = event.paths[0].clone();
                if event.kind.is_create() && is_tar_gz(file.clone()) {
                    unzip_file(file.clone());
                    print!("Unzipped file: {:?}", file);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn is_tar_gz(file: PathBuf) -> bool {
    let regex = Regex::new(r"^artifact_\{.*\}\.tar\.gz$").unwrap();
    regex.is_match(file.file_name().unwrap().to_str().unwrap())
}

fn unzip_or_find(file: PathBuf) {
    let regex = Regex::new(r"^artifact_\{.*\}\.tar\.gz$").unwrap();
    println!("Checking file: {:?}", file);
    if regex.is_match(file.file_name().unwrap().to_str().unwrap()) {
        println!("Unzipping file: {:?}", file);
        match unzip_file(file) {
            Ok(path) => println!("Unzipped file: {:?}", path),
            Err(e) => println!("Error unzipping file: {:?}", e),
        }
    } else {
        println!("Finding index file: {:?}", file);
        match find_index_file(file) {
            Ok(path) => open::that(path.to_str().unwrap()).unwrap(),
            Err(e) => println!("Error finding index file: {:?}", e),
        }
    }
}
use glob::glob;
use regex::Regex;

fn unzip_file(file: PathBuf) -> Result<(), io::Error> {
    let tar_gz = File::open(file.clone())?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let extract_to = file
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No parent directory"))?;
    archive.unpack(extract_to)?;
    println!("Unzipped file1: {:?}", extract_to);
    Ok(())
}

fn find_index_file(file: PathBuf) -> Result<PathBuf, PathBuf> {
    if !file.is_dir() {
        return Err(file);
    }

    for entry in glob(file.join("./**/*index.html").to_str().unwrap()).unwrap() {
        match entry {
            Ok(path) => {
                return Ok(path);
            }
            Err(e) => {
                return Err(file);
            }
        }
    }

    Err(file)
}
