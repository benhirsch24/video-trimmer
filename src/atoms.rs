use parser::*;
use std::fmt;

/* ============================ Traits and helpers ============================ */

pub fn atom_type_and_size(parser: &mut MParserView) -> Result<(u32, String), String> {
    let size = try!(parser.read_u32());
    let typ = try!(TypeParserAction::try_parse(parser));

    Ok((size, typ))
}

fn loop_and_get_children(parser: &mut MParserView, atoms: &[&str]) -> Result<Vec<usize>, String> {
    let mut atom_positions = vec![];

    try!(parser.move_cursor(8));
    loop {
        let (size, typ) = match atom_type_and_size(parser) {
            Ok((s,t)) => (s, t),
            Err(_)    => break
        };

        let actual_pos = parser.get_position() - 8;
        if atoms.contains(&typ.as_str()) {
            atom_positions.push(actual_pos);
            parser.set_position(actual_pos + (size as usize));
        } else {
            break;
        }
    }

    atom_positions.reverse();
    Ok(atom_positions)
}

pub trait AtomParser<T> {
    fn parse(parser: &mut MParserView) -> Result<T, String>;
    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String>;
}

/* ================================= Actual atoms ================================= */

pub struct MovieAtoms {
    pub moov: Option<MoovAtom>,
    pub mvhd: Option<MovieHeaderAtom>,
    pub traks: Vec<TrakAtom>
}

impl MovieAtoms {
    pub fn new() -> MovieAtoms {
        MovieAtoms {
            moov: None,
            mvhd: None,
            traks: vec![]
        }
    }
}

pub struct MoovAtom {
    pub location: usize,
    pub size: u32,
}

impl MoovAtom {
    pub fn new(location: usize, size: u32) -> MoovAtom {
        MoovAtom {
            location: location,
            size: size
        }
    }
}

impl AtomParser<MoovAtom> for MoovAtom {
    fn parse(parser: &mut MParserView) -> Result<MoovAtom, String> {
        let location = parser.get_position();
        let size     = try!(parser.read_u32());

        try!(parser.move_cursor(4));

        Ok(MoovAtom::new(location, size))
    }

    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec!["mvhd", "iods", "trak", "udta"];
        loop_and_get_children(parser, &atoms)
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

impl AtomParser<MovieHeaderAtom> for MovieHeaderAtom {
    fn parse(parser: &mut MParserView) -> Result<MovieHeaderAtom, String> {
        let location                   = parser.get_position();
        let size                       = try!(parser.read_u32());
        try!(parser.move_cursor(4)); // reserved
        let version                    = try!(parser.read_u8());
        let flags                      = try!(parser.read_flags());
        let creation_time              = try!(parser.read_u32());
        let modification_time          = try!(parser.read_u32());
        let time_scale                 = try!(parser.read_u32());
        let duration                   = try!(parser.read_u32());
        let rate                       = try!(parser.read_fixed32());
        let volume                     = try!(parser.read_fixed16());
        try!(parser.move_cursor(36)); // matrix
        let preview_time               = try!(parser.read_u32());
        let preview_duration           = try!(parser.read_u32());
        let poster_time                = try!(parser.read_u32());
        let selection_time             = try!(parser.read_u32());
        let selection_duration         = try!(parser.read_u32());
        let current_time               = try!(parser.read_u32());
        let next_track_id              = try!(parser.read_u32());

        Ok(MovieHeaderAtom::new(location, size, version, flags, creation_time,
                                modification_time, time_scale, duration, rate,
                                volume, preview_time, preview_duration, poster_time,
                                selection_time, selection_duration, current_time, next_track_id))
    }

    fn get_children(_: &mut MParserView) -> Result<Vec<usize>, String> {
        Ok(vec![])
    }
}

pub struct TrakAtom {
    pub location: usize,
    pub size: u32,
    pub tkhd: Option<TrakHeaderAtom>,
    pub mdia: Option<MediaAtom>
}

impl TrakAtom {
    fn new(location: usize, size: u32, tkhd: Option<TrakHeaderAtom>, mdia: Option<MediaAtom>) -> TrakAtom {
        TrakAtom {
            location: location,
            size: size,
            tkhd: tkhd,
            mdia: mdia
        }
    }
}

impl AtomParser<TrakAtom> for TrakAtom {
    fn parse(parser: &mut MParserView) -> Result<TrakAtom, String> {
        let location = parser.get_position();
        let size     = try!(parser.read_u32());
        let mut tkhd = None;
        let mut mdia = None;

        parser.reset();

        let mut atom_position_stack = vec![];
        let mut trak_atom_children = try!(TrakAtom::get_children(parser));
        atom_position_stack.append(&mut trak_atom_children);

        while !atom_position_stack.is_empty() {
            // pop next atom position off stack which should exist
            let stack_pos = atom_position_stack.pop().unwrap();

            // visit (parse the atom)
            let typ = {
                let mut view = parser.get_view_at(stack_pos);

                let (size, typ) = match atom_type_and_size(&mut view) {
                    Ok((s,t)) => (s, t),
                    Err(_)    => break
                };

                println!("   {} @ {} with size {} (stack len = {})", typ, stack_pos, size, atom_position_stack.len());

                match typ.as_str() {
                    "tkhd" => {
                        let atom = try!(TrakHeaderAtom::parse(&mut view));
                        tkhd = Some(atom);
                    },
                    "mdia" => {
                        let atom = try!(MediaAtom::parse(&mut view));
                        mdia = Some(atom);
                    },
                    _      => { println!("Need to parse {}", typ); }
                }

                typ
            };

            // get children and add them to the stack
            {
                let mut view = parser.get_view_at(stack_pos);
                let mut children = match typ.as_str() {
                    "mdia" => try!(MediaAtom::get_children(&mut view)),
                    _      => vec![]
                };
                atom_position_stack.append(&mut children);
            }
        }

        Ok(TrakAtom::new(location, size, tkhd, mdia))
    }

    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec![
            "tkhd", "tapt", "clip", "matt", "edts", "tref",
            "txas", "load", "imap", "mdia", "udta"
        ];

        loop_and_get_children(parser, &atoms)
    }
}

pub struct TrakHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub track_id: u32,
    pub duration: u32,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: f32,
    pub track_width: f32,
    pub track_height: f32
}

impl TrakHeaderAtom {
    fn new(location: usize, size: u32, version: u8, flags: u32, creation_time: u32, modification_time: u32,
           track_id: u32, duration: u32, layer: u16, alternate_group: u16, volume: f32,
           track_width: f32, track_height: f32) -> TrakHeaderAtom {
        TrakHeaderAtom {
            location: location,
            size: size,
            version: version,
            flags: flags,
            creation_time: creation_time,
            modification_time: modification_time,
            track_id: track_id,
            duration: duration,
            layer: layer,
            alternate_group: alternate_group,
            volume: volume,
            track_width: track_width,
            track_height: track_height
        }
    }
}

impl AtomParser<TrakHeaderAtom> for TrakHeaderAtom {
    fn parse(parser: &mut MParserView) -> Result<TrakHeaderAtom, String> {
        let location                  = parser.get_position();
        let size                      = try!(parser.read_u32());
        try!(parser.move_cursor(4));
        let version                   = try!(parser.read_u8());
        let flags                     = try!(parser.read_flags());
        let creation_time             = try!(parser.read_u32());
        let modification_time         = try!(parser.read_u32());
        let track_id                  = try!(parser.read_u32());
        try!(parser.move_cursor(4));  // reserved
        let duration                  = try!(parser.read_u32());
        try!(parser.move_cursor(8));  // reserved
        let layer                     = try!(parser.read_u16());
        let alternate_group           = try!(parser.read_u16());
        let volume                    = try!(parser.read_fixed16());
        try!(parser.move_cursor(2));  // reserved
        try!(parser.move_cursor(36)); // matrix
        let track_width               = try!(parser.read_fixed32());
        let track_height              = try!(parser.read_fixed32());

        Ok(TrakHeaderAtom::new(location, size, version, flags, creation_time, modification_time, track_id,
                               duration, layer, alternate_group, volume, track_width, track_height))
    }

    fn get_children(_: &mut MParserView) -> Result<Vec<usize>, String> {
        Ok(vec![])
    }
}

pub struct MediaAtom {
    pub location: usize,
    pub size: u32,
    pub mdhd: Option<MediaHeaderAtom>,
    pub hdlr: Option<HandlerReferenceAtom>,
    //pub minf: Option<MediaInfoAtom>,
    //pub udta: Option<UserDataAtom>
}

impl MediaAtom {
    fn new(location: usize, size: u32) -> MediaAtom {
        MediaAtom {
            location: location,
            size: size,
            mdhd: None,
            hdlr: None
        }
    }
}

impl AtomParser<MediaAtom> for MediaAtom {
    fn parse(parser: &mut MParserView) -> Result<MediaAtom, String> {
        let location                 = parser.get_position();
        let size                     = try!(parser.read_u32());
        try!(parser.move_cursor(4));

//                    "mdhd" => {
//                        let atom = try!(MediaHeaderAtom::parse(&mut view));
//                        atoms.traks[traknum - 1].mdia.unwrap().mdhd = Some(atom);
//                    },
//                    "hdlr" => {
//                        let atom = try!(HandlerReferenceAtom::parse(&mut view));
//                        atoms.traks[traknum - 1].mdia.unwrap().hdlr = Some(atom);
//                    },

        Ok(MediaAtom::new(location, size))
    }

    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec!["mdhd", "elng", "hdlr", "minf", "udta"];
        loop_and_get_children(parser, &atoms)
    }
}

pub struct MediaHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub time_scale: u32,
    pub duration: u32,
    pub language: u16,
    pub quality: u16
}

impl MediaHeaderAtom {
    fn new( location: usize, size: u32, version: u8, flags: u32, creation_time: u32,
            modification_time: u32, time_scale: u32, duration: u32, language: u16,
            quality: u16) -> MediaHeaderAtom
    {
        MediaHeaderAtom {
            location: location, size: size, version: version, flags: flags,
            creation_time: creation_time, modification_time: modification_time,
            time_scale: time_scale, duration: duration, language: language, quality: quality
        }
    }
}

impl AtomParser<MediaHeaderAtom> for MediaHeaderAtom {
    fn parse(parser: &mut MParserView) -> Result<MediaHeaderAtom, String> {
        let location                 = parser.get_position();
        let size                     = try!(parser.read_u32());
        try!(parser.move_cursor(4));
        let version                  = try!(parser.read_u8());
        let flags                    = try!(parser.read_flags());
        let creation_time            = try!(parser.read_u32());
        let modification_time        = try!(parser.read_u32());
        let time_scale               = try!(parser.read_u32());
        let duration                 = try!(parser.read_u32());
        let language                 = try!(parser.read_u16());
        let quality                  = try!(parser.read_u16());

        Ok(MediaHeaderAtom::new(location, size, version, flags, creation_time,
                                modification_time, time_scale, duration, language,
                                quality))
    }

    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String> {
        Ok(vec![])
    }
}

pub struct HandlerReferenceAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub component_type: u32,
    pub component_subtype: u32,
}

impl HandlerReferenceAtom {
    fn new(location: usize, size: u32, version: u8, flags: u32, component_type: u32,
           component_subtype: u32) -> HandlerReferenceAtom
    {
        HandlerReferenceAtom {
            location: location, size: size, version: version, flags: flags,
            component_type: component_type, component_subtype: component_subtype
        }
    }
}

impl AtomParser<HandlerReferenceAtom> for HandlerReferenceAtom {
    fn parse(parser: &mut MParserView) -> Result<HandlerReferenceAtom, String> {
        let location = parser.get_position();
        let size = try!(parser.read_u32());
        let version = try!(parser.read_u8());
        let flags = try!(parser.read_flags());
        let component_type = try!(parser.read_u32());
        let component_subtype = try!(parser.read_u32());

        Ok(HandlerReferenceAtom::new(location, size, version, flags, component_type, component_subtype))
    }

    fn get_children(parser: &mut MParserView) -> Result<Vec<usize>, String> {
        Ok(vec![])
    }
}
