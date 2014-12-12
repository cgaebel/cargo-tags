#![feature(phase)]
extern crate cargo;
#[phase(plugin, link)] extern crate log;
extern crate serialize;

use std::collections::HashSet;
use std::path::GenericPath;
use std::os;
use cargo::core::{MultiShell, Source};
use cargo::ops;
use cargo::sources::PathSource;
use cargo::util::important_paths::find_root_manifest_for_cwd;
use cargo::util::{mod, Config, CargoResult, BoxError, CliResult, CliError};

const USAGE: &'static str = "
Generate a TAGS file for a local package and all of its dependencies.

Usage:
    cargo tags [options]

Options:
    -h, --help               Print this message
    -e, --emacs              Generate emacs-compatible tags
    --manifest-path PATH     Path to the manifest to compile
    -v, --verbose            Use verbose output
";

#[deriving(Decodable)]
struct Options {
    flag_emacs: bool,
    flag_manifest_path: Option<String>,
    flag_verbose: bool,
}

fn to_display(files: &HashSet<Path>) -> Vec<String> {
    (*files).clone().into_iter().map(|f| f.display().to_string()).collect()
}

fn generate_tags(config: &Config, manifest_path: &Path, options: &Options) -> CargoResult<()> {
    log!(4, "tags; mainfest-path={}", manifest_path.display());

    let source = try!(PathSource::for_path(&manifest_path.dir_path()));
    let packages = try!(source.read_packages());

    log!(4, "tags; packages={}", packages);

    let mut files = HashSet::new();

    // Walk every transitive dependency listed in the lockfile, adding folders
    // with .rs files to the search path list.

    for p in packages.iter() {
        for lockfile in try!(ops::load_pkg_lockfile(p)).into_iter() {
            for node in lockfile.iter() {
                let source_id = node.get_source_id();
                let mut source = source_id.load(config);
                try!(source.update());
                let packages = try!(source.get(&[(*node).clone()]));
                for inner_package in packages.iter() {
                    let manifest_path = inner_package.get_manifest_path();
                    let pathsource = PathSource::new(manifest_path, source_id);
                    let files_in_inner_package = try!(pathsource.list_files(inner_package));
                    let folders = files_in_inner_package.into_iter().map(|f| f.dir_path());
                    files.extend(folders);
                }
            }
        }
    }

    log!(4, "generating tags; paths={}", to_display(&files));

    let mut ctags = try!(util::process("ctags"));

    let rust_cfg: [&'static str, ..11] = include!("ctags.rust");
    let extra_params = [ "--languages=Rust", "--recurse" ];
    let emacs_tags = if options.flag_emacs { Some("-e") } else { None };

    for opt in rust_cfg.iter().chain(extra_params.iter()).chain(emacs_tags.iter()) {
        ctags = ctags.arg(opt);
    }
    for path in files.iter() {
        ctags = ctags.arg(path.display().as_cow().as_slice());
    }

    log!(4, "running ctags; cmd={}", ctags);

    ctags.exec_with_output().map(|_| ()).box_error()
}

fn execute(options: Options, shell: &mut MultiShell) -> CliResult<Option<()>> {
    shell.set_verbose(options.flag_verbose);
    let root = try!(find_root_manifest_for_cwd(options.flag_manifest_path.clone()));
    let config = Config::new(shell, None, None).unwrap();
    generate_tags(&config, &root, &options).map(|_| None).map_err(|err| CliError::from_boxed(err, 101))
}

fn main() {
    cargo::execute_main_with_args_and_without_stdin(
        execute, false, USAGE, os::args().slice_from(1))
}
