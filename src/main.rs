use std::fs;
use std::env;
use std::io::ErrorKind;
use std::process::exit;
use std::{sync::LazyLock};
use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::os::unix::fs::symlink;
use std::process::Command;

static HOME_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let home: String = env::var("HOME").expect("HOME environment variable missing");
    PathBuf::from(home)
});
const TARGET_DIR: &str = ".config";
const DEFAULT_DIR: &str = ".vconfig";
const APPLY_SCRIPT: &str = "apply";


fn create_links(dir_path: &Path) -> Result<(), io::Error> {
    process_source(dir_path, dir_path)
}

fn process_source(dir_path: &Path, source_dir_path: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        if entry.file_name() == APPLY_SCRIPT { continue }
        let entry_path: PathBuf = entry.path();

        if entry_path.is_dir() {
            process_source(&entry_path, source_dir_path)?;

        } else {
            let entry_relative_path: &Path = entry_path.strip_prefix(source_dir_path).expect("Relative path error");
            let target_path: PathBuf = HOME_PATH.join(TARGET_DIR).join(entry_relative_path);

            fs::create_dir_all(target_path.parent().expect("Parent not found"))?;
            if let Err(e) = fs::remove_file(&target_path)
                && e.kind() != ErrorKind::NotFound { eprintln!("{e}: {target_path:?}") }
            symlink(&entry_path, &target_path)?;

            println!("{:?} -> {:?}", target_path, entry_path);
        }
    }
    Ok(())
}

fn apply(dir_path: &Path) {
    let script_path = dir_path.join(APPLY_SCRIPT);
    if script_path.exists() {
        println!("\n==APPLY==");
        let _ = Command::new("bash").arg(script_path).status().expect("Starting apply script error");
    }
}

fn get_source() -> PathBuf {
    let mut relative_path = PathBuf::new();
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        exit(1)
    }
    let mut args_iter = args.into_iter();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-d" | "--dir" => {
                if let Some(val) = args_iter.next() {
                    relative_path = PathBuf::from(val)
                }
            }
            text if !text.starts_with('-') => {
                relative_path = HOME_PATH.join(DEFAULT_DIR).join(text)
            }
            unknown => {
                println!("Unknown arg {unknown}");
                print_usage();
                exit(1)
            }
        }
    }
    match fs::canonicalize(&relative_path) {
        Ok(path) => path,
        Err(e) => {
            println!("{e}: {relative_path:?}");
            exit(1)
        }
    }
}

fn print_usage() {
    println!("Usage:\nvconfig <dirname> (~/{DEFAULT_DIR}/<dirname>)\nvconfig -d <dir>\nvconfig --dir <dir>");
}

fn main() {
    let source: PathBuf = get_source();
    println!("Source: {source:?}");

    create_links(&source).unwrap();
    apply(&source);
}
