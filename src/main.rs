use rust_norg::parse_tree;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let mut input = String::new();

    let file_path = Path::new("test.norg");
    let mut file = match File::open(file_path) {
        Err(why) => panic!("couldn't open {file_path:?}: {why:?}"),
        Ok(file) => file,
    };

    file.read_to_string(&mut input).unwrap();

    println!("{:#?}", parse_tree(&input));
}
