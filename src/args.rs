use super::util::{self, FatalError};
use fs_extra::dir;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "confbk", about = "Easily backup important files")]
pub struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    /// Directory to put the configs in
    out: Option<PathBuf>,

    #[structopt(short, long)]
    /// List files that would be backed up
    dry_run: bool,

    #[structopt(short, long, conflicts_with = "verbose")]
    /// Do not display any output
    quiet: bool,

    #[structopt(short, long, conflicts_with = "quiet")]
    /// Display more verbose output
    verbose: bool,

    #[structopt(short, long, parse(from_os_str), required_unless = "list")]
    /// A file that contains filenames of configs (new line delimited)
    file: Option<PathBuf>,

    #[structopt(short, long, parse(from_os_str), required_unless = "file")]
    /// A list of config files to be backed up
    list: Vec<PathBuf>,

    #[structopt(short, long)]
    /// Compress config dir into a .tar.xz file
    tar: bool,

    #[structopt(short, long, parse(from_os_str))]
    /// Exclude files or directories in the backup
    exclude: Vec<PathBuf>,
}

impl Opt {
    pub fn out(&self) -> &Option<PathBuf> {
        &self.out
    }
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
    pub fn quiet(&self) -> bool {
        self.quiet
    }
    pub fn validate_paths(&self) -> io::Result<Vec<PathBuf>> {
        let mut paths: Vec<PathBuf> = Vec::new();

        // validate files from list
        for path in &self.list {
            if path.is_file() {
                paths.push(path.to_path_buf());
            } else if path.is_dir() {
                let content =
                    dir::get_dir_content(path).expect("Failed to get content of directory");
                let mut content: Vec<PathBuf> = util::all_paths(content);
                paths.append(&mut content);
            } else {
                FatalError::file_not_found(&path.display().to_string());
            }
        }
        // validate files from file
        match &self.file {
            Some(file) => {
                // file exists
                if file.is_file() {
                    let file = File::open(file)?;
                    // line in file
                    for line in BufReader::new(file).lines() {
                        match line {
                            Ok(path) => {
                                // is this a file that exists
                                let path = OsString::from(path);
                                let path = PathBuf::from(path);
                                if path.exists() {
                                    paths.push(path);
                                } else {
                                    FatalError::file_not_found(&path.display().to_string());
                                }
                            }
                            Err(e) => FatalError::error(&e.to_string()),
                        }
                    }
                } else {
                    FatalError::file_not_found(&file.display().to_string());
                }
            }
            // ignore, -f flag was ommitted
            None => (),
        }
        // exclude files from paths
        for excluded in &self.exclude {
            let mut index: i32 = 0;
            while index < (paths.len() as i32) {
                let path = paths.get(index as usize).unwrap();
                let p: String = PathBuf::into_os_string(path.to_path_buf())
                    .to_string_lossy()
                    .to_string();
                let e: String = PathBuf::into_os_string(excluded.to_path_buf())
                    .to_string_lossy()
                    .to_string();

                if p.contains(&e) {
                    paths.remove(index as usize);
                    index -= 1;
                }
                index += 1;
            }
        }
        Ok(paths)
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }
    pub fn tar(&self) -> bool {
        self.tar
    }
    pub fn new() -> Opt {
        Opt::from_args()
    }
}
