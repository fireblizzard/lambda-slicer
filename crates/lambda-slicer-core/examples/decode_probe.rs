//! Development probe that decodes demos and prints the report as JSON

use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    for p in std::env::args().skip(1) {
        let opts = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };

        let t = std::time::Instant::now();
        let (report, _) = decode_demo(p.as_ref(), &opts, |_, _, _| {});
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        eprintln!("{p}: {:.2}s", t.elapsed().as_secs_f64());
    }
}
