use std::error::Error;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ducere::lower::Lower;
use ducere::Reducer;
use tempfile::Builder;

pub fn run(path: PathBuf) {
    match run_(path) {
        Err(e) => panic!("failed {e}"),
        Ok(()) => {}
    }
}

fn run_(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let s = fs::read_to_string(&path)?;
    let file = syn::parse_file(&s)?;
    drop(s);
    let node = file.lower();

    let expected_output = fs::read(path.with_file_name(format!("{file_name}.output")))?;

    let reducer = Reducer {
        root: node,
        rule: ducere::ReduceRule::Fn(Box::new(move |tmp| {
            let prog = tmp.path().parent().unwrap().join("prog");

            if !Command::new("rustc")
                .arg(tmp.path())
                .arg("-o")
                .arg(&prog)
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .status()
                .unwrap()
                .success()
            {
                return false;
            }

            let stdout = Command::new(prog).output().unwrap().stdout;
            stdout == expected_output
        })),
    };

    reducer.reduce()?;

    let mut f = Builder::new()
        .prefix("reducetest")
        .suffix(".rs")
        .tempfile()?;

    let Reducer { root, .. } = reducer;

    write!(f, "{root}")?;

    if !Command::new("rustfmt").arg(f.path()).status()?.success() {
        panic!("failed to run rustfmt");
    }

    let mut found = String::new();
    f.seek(SeekFrom::Start(0))?;
    f.read_to_string(&mut found)?;
    let mut expected_path = path;
    expected_path.set_file_name(format!(
        "{}.reduced",
        expected_path.file_name().unwrap().to_str().unwrap()
    ));
    let expected = fs::read_to_string(&expected_path)?;

    if found != expected {
        panic!("found: {found}, expected: {expected}");
    }

    Ok(())
}
