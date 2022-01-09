use std::error::Error;
use std::path::Path;

use reduceit::lower::Lower;
use reduceit::{Reducer, ReduceRule};


fn main() -> Result<(), std::boxed::Box<dyn Error>> {
    let file = syn::parse_file(r#"pub fn main() -> () {}"#)?;

    let node = file.lower();
    let reducer = Reducer {
        root: node,
        rule: ReduceRule::Program(Path::new("rustc").to_owned()),
    };

    reducer.reduce()?;

    println!("Reduced: {}", reducer.root);

    Ok(())
}
