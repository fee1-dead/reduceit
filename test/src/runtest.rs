use std::error::Error;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use reduceit::Reducer;
use reduceit::lower::Lower;
use tempfile::Builder;

pub fn run(path: PathBuf) {
    match run_(path) {
        Err(e) => panic!("failed {e}"),
        Ok(()) => {}
    }
}

fn run_(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let s = fs::read_to_string(&path)?;
    let file = syn::parse_file(&s)?;
    drop(s);
    let node = file.lower();

    let reducer = Reducer {
        root: node,
        rule: reduceit::ReduceRule::Program(PathBuf::from("rustc")),
    };

    reducer.reduce()?;

    let mut f = Builder::new().prefix("reducetest").suffix(".rs").tempfile()?;

    let Reducer {
        root, ..
    } = reducer;

    write!(f, "{root}")?;

    if !Command::new("rustfmt").arg(f.path()).status()?.success() {
        panic!("failed to run rustfmt");
    }

    let mut found = String::new();
    f.seek(SeekFrom::Start(0))?;
    f.read_to_string(&mut found)?;
    let mut expected_path = path;
    expected_path.set_file_name(format!("{}.reduced", expected_path.file_name().unwrap().to_str().unwrap()));
    let expected = fs::read_to_string(expected_path)?;

    if found != expected {
        panic!("found: {found}, expected: {expected}");
    }
    
    Ok(())
}
