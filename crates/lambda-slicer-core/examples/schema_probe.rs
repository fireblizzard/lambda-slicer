//! Development probe that prints the fields of a delta schema
//!
//! Usage is `schema_probe <demo> <schema>`

use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let name = a.next().unwrap();
    let options = DecodeOptions {
        collect_events: false,
        ..Default::default()
    };
    let (_, d) = decode_demo(path.as_ref(), &options, |_, _, _| {});
    if let Some(s) = d.schema(&name) {
        let fields: Vec<_> = s.fields.iter().map(|f| f.name_lossy()).collect();
        println!("{name}: {}", fields.join(" "));
    }
}
