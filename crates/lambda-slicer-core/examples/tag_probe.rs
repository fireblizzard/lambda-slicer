//! Development probe that prints the outer records of an entry
//!
//! Usage is `tag_probe <demo> <entry> <first ordinal> <count>`

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits, tag};

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let entry: u32 = a.next().unwrap().parse().unwrap();
    let first: u64 = a.next().unwrap().parse().unwrap();
    let count: u64 = a.next().unwrap().parse().unwrap();

    let f = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    for lf in r.frames(entry).unwrap() {
        let lf = lf.unwrap();
        if lf.ordinal < first {
            continue;
        }
        if lf.ordinal >= first + count {
            break;
        }

        let detail = match &lf.frame.body {
            FrameBody::Network(n) => format!("seq {:?} {} bytes", n.sequence, n.message.len()),
            FrameBody::ConsoleCommand(c) => {
                let end = c.iter().position(|&b| b == 0).unwrap_or(c.len());
                String::from_utf8_lossy(&c[..end]).into_owned()
            }
            FrameBody::WeaponAnim { sequence, body } => format!("anim {sequence} body {body}"),
            FrameBody::DemoBuffer(b) => format!("{} bytes", b.len()),
            _ => String::new(),
        };
        println!(
            "#{:<7} {:<14} t {:<12} f {:<8} {}",
            lf.ordinal,
            tag::name(lf.frame.tag),
            lf.frame.time(),
            lf.frame.frame_number,
            detail
        );
    }
}
