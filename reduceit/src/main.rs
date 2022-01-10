use std::error::Error;
use std::path::Path;

use ducere::lower::Lower;
use ducere::{ReduceRule, Reducer};

fn main() -> Result<(), std::boxed::Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let file = syn::parse_file(r#"#![allow(greetings)]
    #![deny(goodbyes)]
    
    pub fn main() -> () {
        if false {
            std::println!("goodbye, world");
        }
    
        if true {
            std::println!("Hello, world!");
        }
    }
    "#)?;

    let node = file.lower();
    let reducer = Reducer {
        root: node,
        rule: ReduceRule::Program(Path::new("rustc").to_owned()),
    };

    reducer.reduce()?;

    println!("Reduced: {}", reducer.root);

    Ok(())
}
