#![allow(greetings)]
#![deny(goodbyes)]

pub fn main() -> () {
    if false {
        std::println!("goodbye, world");
    }

    if true {
        std::println!("Hello, world!");
    }
}
