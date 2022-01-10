//! This is the source code for collecting our tests and running them.
//!
//! The individual test cases can be found in the `test/ui` directory.

// The `test` crate is the only unstable feature
// allowed here, just to share similar code.
#![feature(test)]

use std::env::args;
use std::io;

use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName, TestType};

extern crate test;

mod runtest;

fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    if std::fs::read_to_string("./test/src/main.rs")? != include_str!("main.rs") {
        panic!("ducere tests must be run at the project root.");
    }
    let args: Vec<_> = args().collect();
    let tests = collect_tests()?
        .filter_map(Result::transpose)
        .collect::<io::Result<_>>()?;
    test::test_main(&args, tests, None);

    Ok(())
}

fn collect_tests() -> io::Result<impl Iterator<Item = io::Result<Option<TestDescAndFn>>>> {
    Ok(std::fs::read_dir("./test/ui/")?.map(|dir| match dir {
        Ok(dir) => {
            let path = dir.path();
            if dir.file_type()?.is_file() && path.extension().unwrap_or_default() == "rs" {
                let desc = TestDesc {
                    name: TestName::DynTestName(
                        path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .strip_suffix(".rs")
                            .unwrap()
                            .to_string(),
                    ),
                    ignore: false,
                    should_panic: ShouldPanic::No,
                    allow_fail: false,
                    compile_fail: false,
                    no_run: false,
                    test_type: TestType::UnitTest,
                };

                let testfn = TestFn::DynTestFn(Box::new(move || runtest::run(path)));

                Ok(Some(TestDescAndFn { desc, testfn }))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(e),
    }))
}
