use std::fs::File;
use std::io::Read;
use std::path::Path;

use rust_norg::parse_tree;

fn main() {
    println!("this is the main function");

    let path = Path::new("test.norg");

    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    println!("{:#?}", parse_tree(&s).unwrap());
}
