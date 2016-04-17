use std::env;

extern crate trim;
use trim::videotrim;

fn main() {
    let mut args = env::args().skip(1);

    if args.len() < 3 {
        panic!("Usage: video-trimmer video-file-in video-file-out start stop [moov position]");
    }

    // would expect since we check the length these args exist
    let video_in = args.nth(0).unwrap();
    let video_out = args.nth(0).unwrap();
    let start = args.nth(0).unwrap().parse::<f32>().expect("Start was not an f32");
    let stop = args.nth(0).unwrap().parse::<f32>().expect("Stop was not an f32");

    match videotrim::trim_video(&video_in, &video_out, start, stop) {
        Ok(_)  => println!("Video trimmed!"),
        Err(e) => println!("Error while trimming video: {}", e)
    };
}
