#![allow(greetings)]
#![deny(goodbyes)]

pub fn main() -> () {
    if false {
        std::println!("goodbye, world");
    } else {
        std::print!("Hello, ");
    }

    if true {
        1 + 1;
        std::println!("world!");
        2 + 2;
    }
}
