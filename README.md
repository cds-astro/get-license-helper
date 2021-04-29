[![](https://meritbadge.herokuapp.com/get-license-helper)](https://crates.io/crates/get-license-helper)
[![](https://img.shields.io/crates/d/get-license-helper.svg)](https://crates.io/crates/get-license-helper)


Get Licenses Helper
===================

Uses the JSON output of [cargo-license](https://github.com/onur/cargo-license) (`cargo license --json`)
to try to find and automatically download your Rust project dependencies license files.  

It is based on an heuristic approach, so you may have to download manually licenses not found automatically
(especially if a crate does not come from crates.io).


Use case
--------

Help in bundling licenses when creating a `conda-forge` recipe.  
See e.g. the `license_file` section of `meta.yaml` and the `library_licenses` directory in
[cdshealpix-feedstock](https://github.com/conda-forge/cdshealpix-feedstock/tree/master/recipe),
(supposedly) mimicking [wasmer-feedstock](https://github.com/conda-forge/wasmer-feedstock/tree/682a7bf5ac5e723176e3a34fc32880b4adcae022/recipe).


Install
-------

First, requires [cargo-license](https://github.com/onur/cargo-license) :
```bash
cargo install cargo-license
```

Then, install either from a local clone of the repository:
```bash
cargo install --path .
```
or direclty from [crates.io](https://crates.io/):
```bash
cargo install get-license-helper
```

Usage
-----
Result of `get-license-helper  --help`:
```bash
get-license-helper 0.1.0
Help in downloading license files from the cargo-license --json output.

USAGE:
    get-license-helper [OPTIONS] [input]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l <license-dir>        Directory storing the licenses [default: library_licenses]

ARGS:
    <input>    Input file (result of cargo-license --json), stdin if not present
```

Remark: using twice the same command is faster since already downladed licenses are not re-downloaded.

Example
-------

In your project directory type:
```bash
cargo license --json | get-license-helper
```
and check the content of the `library_licenses` directory.

You can obviously re-direct the output:
```bash
cargo license --json | get-license-helper > list.yaml
```

Change the downloaded licenses output directory:
```bash
cargo license --json | get-license-helper -l my_licenses > list.yaml
```

Or use the saved result of `cargo-license`:
```bash
cargo license --json > licenses.json
get-license-helper licenses.json -l my_licenses > list.yaml
```


Heuristic approach
------------------

Ideally, the heuristic should first search in `https://docs.rs/crate/${name}/${version}/source/`,
but I don't know if one can download the raw files instead of HTML pages.  
So far, we rely on the `repositories`:
* look at the `cargo-license` provided `repository`:
    + if `github`: look for license(s) in `https://raw.githubusercontent.com/${repo}`
        - with `${repo}` = `${repository}` removing starting `https://github.com/` and possible ending `.git`
    + if `gitlab`: look for license(s) in `${repository}/-/raw` removing possible ending `.git` to `${repository}`
    + else, emits a warning
* for the name of the license file(s):
    + look for the `cargo-license` provided `license` (split on ' OR ' in case of multiple licenses):
        - "MIT": look for `LICENSE-MIT(.txt|.md)` or `LICENSE(.txt|.md)`
        - "Apache-2.0": look for `LICENSE-APACHE(.txt|.md)`, `LICENSE-Apache(.txt|.md)` or `LICENSE(.txt|.md)`
        - "BSD-3-Clause": look for `LICENSE-BSD(.txt|.md)` or `LICENSE(.txt|.md)`
        - "BSD-2-Clause": look for `LICENSE-BSD(.txt|.md)` or `LICENSE(.txt|.md)`
        - "ISC":  look for `LICENSE-ISC(.txt|.md)` or `LICENSE(.txt|.md)`
        - "BSL-1.0": look for `LICENSE-BOOST(.txt|.md)`, `LICENSE-BST(.txt|.md)` or `LICENSE(.txt|.md)`
        - "Unlicense": do nothing
    + if no `license`, look at the `cargo-license` provided `license-file`
        - if not found, still look for `LICENSE(.txt|.md)`

To Be Impoved
--------------

* [ ] Performances: implement a pool of `async` workers to explore (HTTP queries) `n` crates at the same time.
* [ ] Add more licenses: to be done when new cases shows up.
* [ ] ? Feature: check that license files are not empty and are in agreement with the declared license type
* [ ] ? Feature: automatically create a LICENSE file (from a given templates) if license not found
* [ ] ...

Warning
--------

* Using sequential blocking HTTP requests, this is slow.
* Create duplicate output lines if the same crate (with a different versions) is use multiple times
* Do not check that the license text is in agreement with the declared license

License
-------

Like most projects in Rust, this project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.


Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.







