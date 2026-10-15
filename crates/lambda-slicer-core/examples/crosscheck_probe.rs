//! Development probe: run decode cross-checks and print the report.
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::validate::crosscheck::crosscheck_demo;

fn main() {
    for p in std::env::args().skip(1) {
        let r = crosscheck_demo(p.as_ref(), Limits::default());
        println!(
            "{} (error {:?}, inferred {}, spectator frames skipped {})",
            r.path, r.decode_error, r.inferred_delta_bases, r.skipped_spectator_frames
        );
        for c in &r.exact {
            println!(
                "  {:>8}/{:<8} before {:>8} after {:>8}  {}",
                c.match_either, c.compared, c.match_before, c.match_after, c.name
            );
            for e in &c.examples {
                println!(
                    "      e{} f{} t{} recorded {} before {} after {}",
                    e.entry, e.frame_number, e.time, e.recorded, e.before, e.after
                );
            }
        }
        for c in &r.distance {
            println!(
                "  {:>8} max {:<10.4} bins<= {:?}: {:?}  {}",
                c.compared, c.max, c.bins, c.counts, c.name
            );
            for o in &c.outliers {
                println!(
                    "      outlier e{} f{} t{} distance {:.4}",
                    o.0, o.1, o.2, o.3
                );
            }
        }
    }
}
