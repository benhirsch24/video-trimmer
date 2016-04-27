use atoms::*;
use parser::*;
use std::str;


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
                let moov_pos = i - 3;

                let mut chars = &mut [0; 4];
                chars[0] = data[moov_pos];
                chars[1] = data[moov_pos + 1];
                chars[2] = data[moov_pos + 2];
                chars[3] = data[moov_pos + 3];
                println!("Found moov atom at {}, sanity check: {}",
                         moov_pos,
                         str::from_utf8(chars).unwrap());

                return Ok(moov_pos - 4);
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
    let mut atom_position_stack = vec![moov_pos];

    let mut atoms = MovieAtoms::new();
    while atom_position_stack.len() > 0
    {
        // pop next atom position off stack which should exist
        let stack_pos = atom_position_stack.pop().unwrap();

        // visit (parse the atom)
        let typ = {
            let position = parser.get_position();
            let mut view = parser.get_view_at(stack_pos);

            let (size, typ) = match atom_type_and_size(&mut view) {
                Ok((s,t)) => (s, t),
                Err(_)    => break
            };

            println!("{} @ {} with size {} (stack len = {})", typ, stack_pos, size, atom_position_stack.len());

            match typ.as_str() {
                "moov" => {
                    let atom = Some(MoovAtom::new(position, size));
                    atoms.moov = atom;
                },
                "mvhd" => {
                    let atom = Some(try!(MovieHeaderAtom::parse(size, &mut view)));
                    atoms.mvhd = atom;
                },
                _      => { println!("Haven't yet implemented {}", typ); }
            };

            typ
        };

        // get children and add them to the stack
        {
            let mut view = parser.get_view_at(stack_pos);
            let mut children = match typ.as_str() {
                "moov" => try!(MoovAtom::get_children(&mut view)),
                "mvhd" => try!(MovieHeaderAtom::get_children(&mut view)),
                _      => { println!("Haven't yet implemented {}", typ); vec![] }
            };
            atom_position_stack.append(&mut children);
        }
    };

    Ok(())
}

