
use structopt::StructOpt;

use serde::{Deserialize};

use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::{Path, PathBuf};
use std::io::Read;

const DEFAULT: [&str; 1] = ["LICENSE"];
const MIT: [&str; 1] = ["LICENSE-MIT"];
const APACHE: [&str; 2] = ["LICENSE-APACHE", "LICENSE-Apache"];
const BSD3: [&str; 1] = ["LICENSE-BSD"];
const BSD2: [&str; 1] = ["LICENSE-BSD"];
const ISC: [&str; 1] = ["LICENSE-ISC"];
const BSL: [&str; 2] = ["LICENSE-BOOST", "LICENSE-BST"];

#[derive(Debug, StructOpt)]
#[structopt(name = "get-license-helper", about = "Help in downloading license files from the cargo-license --json output.")]
struct Args {
  /// Input file (result of cargo-license --json), stdin if not present
  #[structopt(parse(from_os_str))]
  input: Option<PathBuf>,
  /// Directory storing the licenses
  #[structopt(short = "l", parse(from_os_str), default_value = "library_licenses")]
  license_dir: PathBuf
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

fn get_license(elem: &Elem, base_url: &str, license: &[&str], output_dir: &Path) -> Result<(), Box<dyn Error>> {
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
    let license_names: Vec<String> = license_names.iter().map(
      |l| vec![l.to_string(), format!("{}.txt", l), format!("{}.md", l)]
    ).flatten().collect();
    // Try first with the version as a tag, else look at the master branch
    let versions = elem.version.as_ref().map(|v| vec![v.as_str(), "master"])
      .unwrap_or_else(|| vec!["master"]);
    'outer: for license_name in license_names {
      for v in versions.iter() {
        let url = format!("{}/{}/{}", base_url, v, license_name);
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
    println!("{} not found for crate {}. See repo: {}", license[0], elem.name, elem.repository.as_ref().unwrap());
  } else {
    println!("    - {}", local_path.to_string_lossy());
  }
  Ok(())
}

fn get_raw_files_url(repo_url: &str) -> Option<String> {
  let repo_url = repo_url.trim_end_matches(".git");
  if repo_url.starts_with("https://gitlab.") {
    Some(format!("{}/-/raw", repo_url))
  } else if repo_url.starts_with("https://github.com/") {
    Some(
      format!("https://raw.githubusercontent.com/{}",
        repo_url.strip_prefix("https://github.com/").unwrap()
      )
    )
  } else {
    None
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
    let repo_url = e.repository.as_ref().cloned().unwrap_or(format!("No repo for crate {}!", e.name));
    match get_raw_files_url(&repo_url) {
      Some(url_raw) => {
        match e.license.as_ref() {
          Some(license) =>
            for l in license.split(" OR ") {
              // TODO: list to be completed!
              match l {
                "Apache-2.0" | "Apache-2.0 WITH LLVM-exception" => get_license(&e, &url_raw, &APACHE, &args.license_dir)?,
                "MIT" => get_license(&e, &url_raw, &MIT, &args.license_dir)?,
                "BSD-3-Clause" => get_license(&e, &url_raw, &BSD3, &args.license_dir)?,
                "BSD-2-Clause" => get_license(&e, &url_raw, &BSD2, &args.license_dir)?,
                "BSD" => get_license(&e, &url_raw, &BSD3, &args.license_dir)?,
                "ISC" => get_license(&e, &url_raw, &ISC, &args.license_dir)?,
                "BSL-1.0" => get_license(&e, &url_raw, &BSL, &args.license_dir)?,
                "Unlicense" => { /* No license, do nothing. */ },
                _ if l.starts_with("Apache-2.0") => get_license(&e, &url_raw, &APACHE, &args.license_dir)?,
                _ => println!("Not implemented: license: {}, see repo: {}", l, repo_url),
              }
            },
          None => {
            let license = &e.license_file.as_ref().map(|s| [s.as_str()]).unwrap_or(DEFAULT);
            get_license(&e, &url_raw, license, &args.license_dir)?;
          },
        }
      },
      None  => println!("Unfamiliar repository URL: {}", repo_url),
    }
  }
  Ok(())
}
