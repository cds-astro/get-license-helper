use structopt::StructOpt;

use serde::Deserialize;

use std::error::Error;
use std::fs::{self, read_to_string};
use std::io::Read;
use std::path::{Path, PathBuf};

// Add licence everywhere.
const DEFAULT: [&str; 2] = ["LICENSE", "LICENCE"];
const APACHE: [&str; 6] = [
    "LICENSE-APACHE",
    "LICENSE-Apache",
    "License-Apache-2.0",
    "LICENCE-APACHE",
    "LICENCE-Apache",
    "Licence-Apache-2.0",
];
const BSD2: [&str; 2] = ["LICENSE-BSD", "LICENCE-BSD"];
const BSD3: [&str; 2] = ["LICENSE-BSD", "LICENCE-BSD"];
const BSL: [&str; 4] = [
    "LICENSE-BOOST",
    "LICENSE-BST",
    "LICENCE-BOOST",
    "LICENCE-BST",
];
const CC0: [&str; 2] = ["LICENSE", "LICENCE"];
const ISC: [&str; 2] = ["LICENSE-ISC", "LICENCE-ISC"];
const MIT: [&str; 2] = ["LICENSE-MIT", "LICENCE-MIT"];
const MPL_2: [&str; 2] = ["LICENSE", "LICENCE"];
const ZERO_BSD: [&str; 2] = ["LICENSE-0BSD", "LICENCE-0BSD"];
const ZLIB: [&str; 2] = ["LICENSE-ZLIB", "LICENCE-ZLIB"];

#[derive(Debug, StructOpt)]
#[structopt(
    name = "get-license-helper",
    about = "Help in downloading license files from the cargo-license --json output."
)]
struct Args {
    /// Input file (result of cargo-license --json), stdin if not present
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
    /// Directory storing the licenses
    #[structopt(short = "l", parse(from_os_str), default_value = "library_licenses")]
    license_dir: PathBuf,
}

#[derive(Deserialize)]
struct Elem {
    name: String,
    version: Option<String>,
    //authors: Option<String>,
    repository: Option<String>,
    license: Option<String>,
    license_file: Option<String>,
    //description: Option<String>
}

fn get_license(
    elem: &Elem,
    base_url: &RawFilesURL,
    license: &[&str],
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    assert!(!license.is_empty());
    fs::create_dir_all(output_dir)?;
    let local_path = output_dir.join(format!("{}-{}", elem.name, license[0]));
    // Check if the license file has already been downloaded
    let mut success = local_path.is_file() && fs::metadata(&local_path)?.len() > 0;
    if !success {
        // Try first with the provided license name (e.g. LICENSE-MIT(.txt|.md)),
        // then with the generic "LICENSE(.txt|.md)"
        let mut license_names = license.to_vec();
        // Add default names if not already the default
        if license != DEFAULT {
            license_names.extend_from_slice(&DEFAULT);
        }
        // Add extensions '.txt' and '.md'
        let license_names: Vec<String> = license_names
            .iter()
            .map(|l| {
                vec![
                    l.to_string(),
                    format!("{}.txt", l),
                    format!("{}.md", l),
                    l.to_lowercase(),
                    format!("{}.txt", l.to_lowercase()),
                    format!("{}.md", l.to_lowercase()),
                ]
            })
            .flatten()
            .collect();
        // Try first with the version as a tag, else look at the master and main branches.
        let versions = elem
            .version
            .as_ref()
            .map(|version| vec![version.as_str(), "master", "main"])
            .unwrap_or_else(|| vec!["master", "main"]);
        'outer: for license_name in license_names {
            for version in versions.iter() {
                let url = base_url.format(version, &license_name);
                let mut resp = reqwest::blocking::get(&url)?;
                if resp.status().is_success() {
                    let mut file = std::fs::File::create(&local_path)?;
                    resp.copy_to(&mut file)?;
                    success = true;
                    break 'outer;
                }
            }
        }
    }
    if !success {
        println!(
            "{} not found for crate {}. See repo: {}",
            license[0],
            elem.name,
            elem.repository.as_ref().unwrap()
        );
    } else {
        println!("    - {}", local_path.to_string_lossy());
    }
    Ok(())
}

struct RawFilesURL {
    base: String,
    subdirectory: Option<String>,
}

impl RawFilesURL {
    fn from_repo_url(repo_url: &str) -> Option<Self> {
        let repo_url = repo_url.trim_end_matches(".git");
        if repo_url.starts_with("https://gitlab.") {
            Some(Self {
                base: format!("{}/-/raw", repo_url),
                subdirectory: None,
            })
        } else if repo_url.starts_with("https://github.com/") {
            if let Some((repo_url, path)) = repo_url.split_once("/tree/master/") {
                // Ex) "https://github.com/clap-rs/clap/tree/master/clap_lex"
                Some(Self {
                    base: format!(
                        "https://raw.githubusercontent.com/{}",
                        repo_url.strip_prefix("https://github.com/").unwrap(),
                    ),
                    subdirectory: Some(path.to_string()),
                })
            } else {
                // Ex) https://github.com/plotters-rs/plotters.git
                Some(Self {
                    base: format!(
                        "https://raw.githubusercontent.com/{}",
                        repo_url
                            .strip_prefix("https://github.com/")
                            .unwrap()
                    ),
                    subdirectory: None,
                })
            }
        } else {
            None
        }
    }

    fn format(&self, version: &str, filename: &str) -> String {
        if let Some(subdirectory) = &self.subdirectory {
            format!("{}/{}/{}/{}", self.base, version, subdirectory, filename)
        } else {
            format!("{}/{}/{}", self.base, version, filename)
        }
    }
}

// Put the content of the input file (or stdin) in a string
fn get_input_data_as_string(args: &Args) -> std::io::Result<String> {
    match &args.input {
        Some(path) => read_to_string(path),
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();
    // Load the full JSON at once
    let data = get_input_data_as_string(&args)?;
    // Deserialize to obtain a vector of objects (one per crate)
    let dependencies: Vec<Elem> = serde_json::from_str(&data)?;
    // TODO: create a pool of async workers to process n repo at "the same" time
    for e in dependencies {
        let repo_url = e
            .repository
            .as_ref()
            .cloned()
            .unwrap_or(format!("No repo for crate {}!", e.name));
        match RawFilesURL::from_repo_url(&repo_url) {
            Some(url_raw) => {
                match e.license.as_ref() {
                    Some(license) => {
                        for l in license.split(" OR ") {
                            // TODO: list to be completed!
                            match l {
                                "Apache-2.0" | "Apache-2.0 WITH LLVM-exception" => {
                                    get_license(&e, &url_raw, &APACHE, &args.license_dir)?
                                }
                                "MIT" => get_license(&e, &url_raw, &MIT, &args.license_dir)?,
                                "BSD-3-Clause" => {
                                    get_license(&e, &url_raw, &BSD3, &args.license_dir)?
                                }
                                "BSD-2-Clause" => {
                                    get_license(&e, &url_raw, &BSD2, &args.license_dir)?
                                }
                                "0BSD" => get_license(&e, &url_raw, &ZERO_BSD, &args.license_dir)?,
                                "CC0-1.0" => get_license(&e, &url_raw, &CC0, &args.license_dir)?,
                                "MPL-2.0" => get_license(&e, &url_raw, &MPL_2, &args.license_dir)?,
                                "BSD" => get_license(&e, &url_raw, &BSD3, &args.license_dir)?,
                                "ISC" => get_license(&e, &url_raw, &ISC, &args.license_dir)?,
                                "BSL-1.0" => get_license(&e, &url_raw, &BSL, &args.license_dir)?,
                                "Zlib" => get_license(&e, &url_raw, &ZLIB, &args.license_dir)?,
                                "Unlicense" => { /* No license, do nothing. */ }
                                _ if l.starts_with("Apache-2.0") => {
                                    get_license(&e, &url_raw, &APACHE, &args.license_dir)?
                                }
                                _ => println!(
                                    "Not implemented: license: {}, see repo: {}",
                                    l, repo_url
                                ),
                            }
                        }
                    }
                    None => match &e.license_file {
                        Some(license) => {
                            get_license(&e, &url_raw, &[license.as_str()], &args.license_dir)?
                        }
                        None => get_license(&e, &url_raw, &DEFAULT, &args.license_dir)?,
                    },
                }
            }
            None => println!("Unfamiliar repository URL: {}", repo_url),
        }
    }
    Ok(())
}
