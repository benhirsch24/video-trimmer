use atoms::*;
use parser::MParser;

pub fn trim_video(filename: &String, start: f32, stop: f32, moov: Option<usize>) {
    println!("Trimming {} from {} to {}", filename, start, stop);

    let mut parser = match MParser::new(filename) {
        Ok(p)  => p,
        Err(e) => panic!(e)
    };

    let moov_atom = match MoovAtom::parse(&mut parser) {
        Ok(a) => a,
        Err(e) => panic!(e)
    };

    println!("Moov atom has size: {}", moov_atom.size);
}

