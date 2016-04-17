use atoms::*;
use parser::*;
use std::str;

fn atom_type_and_size(parser: &mut MParser) -> Result<(u32, String), String> {
    let size = try!(parser.read_u32());
    let typ = try!(TypeParserAction::try_parse(parser));

    Ok((size, typ))
}


fn find_moov(data: &[u8]) -> Result<usize, String> {
    let mut m = 0;
    for i in 0..data.len() {
        match data[i] as char {
            'm' => if m == 0 {
                m = m + 1;
            } else {
                m = 0;
            },
            'o' => if m == 1 || m == 2 {
                m = m + 1;
            } else {
                m = 0;
            },
            'v' => if m == 3 {
                println!("v @ {}", i);
                return Ok(i - 3);
            } else {
                m = 0;
            },
            _ => m = 0
        }
    }

    Err(format!("Could not find moov atom"))
}


pub fn trim_video(video_in: &str, video_out: &str, start: f32, stop: f32) -> Result<(), String> {
    println!("Trimming {} into {} from {} to {}", video_in, video_out, start, stop);

    let mut parser = match MParser::new(video_in) {
        Ok(p)  => p,
        Err(e) => panic!(e)
    };

    let moov_pos = try!(find_moov(&parser.data));
    let moov_atom_start = moov_pos - 4;

    let mut chars = &mut [0; 4];
    chars[0] = parser.data[moov_pos];
    chars[1] = parser.data[moov_pos + 1];
    chars[2] = parser.data[moov_pos + 2];
    chars[3] = parser.data[moov_pos + 3];
    println!("Found moov atom at {}, sanity check: {}",
             moov_atom_start,
             str::from_utf8(chars).unwrap());
    parser.set_position(moov_atom_start);

    let (mut atoms_size, typ) = match atom_type_and_size(&mut parser) {
        Ok((s,t)) => (s, t),
        Err(e)    => return Err(e)
    };

    if typ != "moov".to_string() {
        return Err(format!("Expected \"moov\" found {}", typ));
    }

    let mut atoms = MovieAtoms::new();
    while atoms_size > 0 && parser.get_position() < parser.get_size() {
        let parser_position = parser.get_position();

        let (size, typ) = match atom_type_and_size(&mut parser) {
            Ok((s,t)) => (s, t),
            Err(e)    => return Err(e)
        };

        match typ.as_str() {
            "moov" => atoms.moov = Some(MoovAtom::new(parser.get_position(), atoms_size)),
            "mvhd" => atoms.mvhd = Some(try!(MovieHeaderAtom::parse(size, &mut parser))),
            _      => { println!("Haven't yet parsed {}", typ) }
        };

        parser.set_position(parser_position + size as usize);

        println!("atoms_size: {}, new atom size: {}, type: {}, parser_pos: {}, parser_size: {}", atoms_size, size, typ, parser.get_position(), parser.get_size());

        atoms_size -= size;
    };

    Ok(())
}

