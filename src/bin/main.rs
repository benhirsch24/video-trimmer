use std::env;

extern crate trim;
use trim::videotrim;

fn main() {
    let mut args = env::args().skip(1);

    if args.len() < 3 {
        panic!("Usage: video-trimmer video-file start stop [moov position]");
    }

    // would expect since we check the length these args exist
    let video = args.nth(0).unwrap();
    let start = args.nth(0).unwrap().parse::<f32>().expect("Start was not an f32");
    let stop = args.nth(0).unwrap().parse::<f32>().expect("Stop was not an f32");

    let moov = match args.nth(0) {
        Some(m) => Some(m.parse::<usize>().expect("Moov was not a usize")),
        None    => None
    };

    videotrim::trim_video(&video, start, stop, moov);
}
