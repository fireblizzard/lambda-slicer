//! Development survey of the records in a demo's LOADING entry, with their
//! tag, time, frame number and kind

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use lambda_slicer_core::container::{DemoReader, Limits};

fn main() {
    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(arg);
        let file = BufReader::new(File::open(&path).unwrap());
        let mut reader = DemoReader::new(file, Limits::default()).unwrap();
        println!("{}", path.display());
        for item in reader.frames(0).unwrap() {
            let f = item.unwrap().frame;
            let kind = format!("{:?}", f.body);
            let kind: String = kind.chars().take_while(|c| c.is_alphanumeric()).collect();
            println!(
                "  tag {} time {:.3} frame {} {kind}",
                f.tag,
                f.time(),
                f.frame_number
            );
        }
    }
}
