use parser::*;
use std::fmt;

pub trait AtomParser<T> {
    fn parse(parser: &mut MParser) -> Result<T, String>;
}

#[derive(Debug)]
pub struct MovieHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub time_scale: u32,
    pub duration: u32,
    pub rate: f32,
    pub volume: f32,
    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32
}

impl MovieHeaderAtom {
    fn new(location: usize, size: u32, version: u8, flags: u32,
           creation_time: u32, modification_time: u32, time_scale: u32,
           duration: u32, rate: f32, volume: f32,
           preview_time: u32, preview_duration: u32, poster_time: u32,
           selection_time: u32, selection_duration: u32, current_time: u32,
           next_track_id: u32) -> MovieHeaderAtom
    {
        MovieHeaderAtom {
            location: location,
            size: size,
            version: version,
            flags: flags,
            creation_time: creation_time,
            modification_time: modification_time,
            time_scale: time_scale,
            duration: duration,
            rate: rate,
            volume: volume,
            preview_time: preview_time,
            preview_duration: preview_duration,
            poster_time: poster_time,
            selection_time: selection_time,
            selection_duration: selection_duration,
            current_time: current_time,
            next_track_id: next_track_id
        }
    }
}

fn atom_type_and_size(atom: &str, parser: &mut MParser) -> Result<u32, String> {
    let size = try!(parser.read_u32());
    let typ = try!(TypeParserAction::try_parse(parser));
    if typ != atom.to_string() {
        return Err(format!("Type of atom didn't match; expected \"{}\" got {}", atom, typ));
    }

    Ok(size)
}

impl AtomParser<MovieHeaderAtom> for MovieHeaderAtom {
    fn parse(parser: &mut MParser) -> Result<MovieHeaderAtom, String> {
        let location = parser.get_position();

        let size = try!(atom_type_and_size("mvhd", parser));

        let version = try!(parser.read_u8());
        let flags = try!(parser.read_flags());
        let creation_time = try!(parser.read_u32());
        let modification_time = try!(parser.read_u32());
        let time_scale = try!(parser.read_u32());
        let duration = try!(parser.read_u32());
        let rate = try!(parser.read_fixed32());
        let volume = try!(parser.read_fixed16());

        // matrix advances by 36 bytes
        parser.move_cursor(36);

        let preview_time = try!(parser.read_u32());
        let preview_duration = try!(parser.read_u32());
        let poster_time = try!(parser.read_u32());
        let selection_time = try!(parser.read_u32());
        let selection_duration = try!(parser.read_u32());
        let current_time = try!(parser.read_u32());
        let next_track_id = try!(parser.read_u32());

        Ok(MovieHeaderAtom::new(
            location,
            size,
            version,
            flags,
            creation_time,
            modification_time,
            time_scale,
            duration,
            rate,
            volume,
            preview_time,
            preview_duration,
            poster_time,
            selection_time,
            selection_duration,
            current_time,
            next_track_id
                ))
    }
}

pub struct MoovAtom {
    pub location: usize,
    pub size: u32,
    pub mvhd: Option<MovieHeaderAtom>
}

impl MoovAtom {
    fn new(location: usize, size: u32, mvhd: Option<MovieHeaderAtom>) -> MoovAtom {
        MoovAtom {
            location: location,
            size: size,
            mvhd: mvhd
        }
    }
}

impl fmt::Display for MoovAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom moov @ {} of size {}; ends at {}",
               self.location,
               self.size,
               (self.location as u32) + self.size)
    }
}

macro_rules! parse_atom {
    ( $atom:ident, $parser:ident ) => {
        {
            //$parser.push_stack();
            try!(
                match $atom::parse($parser) {
                    Ok(a)  => Ok(Some(a)),
                    Err(e) => {
                        //$parser.unwind();
                        Err(e)
                    }
                }
            )
        }
    }
}

impl AtomParser<MoovAtom> for MoovAtom {
    fn parse(parser: &mut MParser) -> Result<MoovAtom, String> {
        let location = parser.get_position();
        let size = try!(atom_type_and_size("moov", parser));

        let mvhd_atom = parse_atom!(MovieHeaderAtom, parser);

        return Ok(MoovAtom::new(location, size, mvhd_atom));
    }
}
