extern crate roxmltree;
extern crate zip;

use std::io::Read;

#[derive(Debug)]
pub struct Song {
    pub global_song_data: GlobalSongData,
    pub tracks: Vec<Track>,
    pub pattern_pool: PatternPool,
    pub pattern_sequence: PatternSequence
}

impl Song {
    pub fn from_xml(xml: String) -> Result<Song, Box<dyn std::error::Error>> {
        let doc = match roxmltree::Document::parse(&xml) {
            Ok(doc) => doc,
            Err(e) => { bail!(e); }
        };

        Ok(
            Song {
                global_song_data: make_global_song_data(&doc),
                tracks: collect_tracks(&doc),
                pattern_pool: make_pattern_pool(&doc),
                pattern_sequence: make_pattern_sequence(&doc)
            }
        )
    }

    pub fn from_xrns(xrns: &std::path::Path) -> Result<Song, Box<dyn std::error::Error>> {
        let song_file = std::fs::File::open(&xrns).unwrap();
        let mut archive = zip::ZipArchive::new(song_file).unwrap();

        let mut file = match archive.by_name("Song.xml") {
            Ok(file) => file,
            Err(_) => { bail!("Not a valid Renoise song"); }
        };

        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        Song::from_xml(data)
    }
}

#[derive(Debug)]
pub struct GlobalSongData {
    pub beats_per_min: u32,
    pub lines_per_beat: u32,
    pub ticks_per_line: u32,

    pub song_name: String,
    pub artist: String
}

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub color: String,
    pub state: String,

    pub number_of_visible_note_columns: u32,
    pub number_of_visible_effect_columns: u32,
    pub volume_column_is_visible: bool,
    pub panning_column_is_visible: bool,
    pub delay_column_is_visible: bool,
}

#[derive(Debug)]
pub struct PatternPool {
    pub patterns: Vec<Pattern>
}

#[derive(Debug)]
pub struct Pattern {
    pub number_of_lines: u32,
    pub tracks: Vec<PatternTrack>
}

#[derive(Debug)]
pub struct PatternTrack {
    pub r#type: String,
    pub alias_pattern_index: i32,

    pub lines: Vec<Line>
}

#[derive(Debug)]
pub struct Line {
    pub index: u32,

    pub note_columns: Vec<Option<NoteColumn>>,
    pub effect_columns: Vec<Option<EffectColumn>>
}

#[derive(Debug)]
pub struct NoteColumn {
    pub note: String,
    pub instrument: String,
    pub volume: String,
    pub panning: String
}

#[derive(Debug)]
pub struct EffectColumn {
    pub value: String,
    pub number: String
}

#[derive(Debug)]
pub struct PatternSequence {
    pub sequence_entries: Vec<SequenceEntry>
}

#[derive(Debug)]
pub struct SequenceEntry {
    pub pattern: u32,
    pub section_name: String,
    pub muted_tracks: Vec<u32>
}

macro_rules! find_tag {
    ($doc:expr, $name:expr) => {
        $doc.children().find(|n| n.has_tag_name($name))
    }
}
macro_rules! find_tag_text {
    ($doc:expr, $name:expr) => {
        find_tag!($doc, $name).unwrap().text()
    };
    ($doc:expr, $name:expr, $default:expr) => {
        match find_tag!($doc, $name) {
            Some(n) => { n.text() },
            None => { Some($default) }
        }
    }
}
macro_rules! find_tag_text_parsed {
    ($doc:expr, $name:expr) => {
        find_tag_text!($doc, $name).unwrap().parse()
    }
}

fn make_global_song_data(doc: &roxmltree::Document) -> GlobalSongData {
    let gsd = find_tag!(doc.root_element(), "GlobalSongData").unwrap();

    GlobalSongData {
        beats_per_min: find_tag_text_parsed!(gsd, "BeatsPerMin").unwrap(),
        lines_per_beat: find_tag_text_parsed!(gsd, "LinesPerBeat").unwrap(),
        ticks_per_line: find_tag_text_parsed!(gsd, "TicksPerLine").unwrap(),

        song_name: find_tag_text!(gsd, "SongName").unwrap().to_string(),
        artist: find_tag_text!(gsd, "Artist").unwrap().to_string()
    }
}

fn collect_tracks(doc: &roxmltree::Document) -> Vec<Track> {
    let tracks = find_tag!(doc.root_element(), "Tracks").unwrap();

    tracks.children().filter(|n| n.is_element()).map(|n|
        Track {
            name: find_tag_text!(n, "Name").unwrap().to_string(),
            color: find_tag_text!(n, "Color").unwrap().to_string(),
            state: find_tag_text!(n, "State").unwrap().to_string(),
            number_of_visible_note_columns: find_tag_text_parsed!(n, "NumberOfVisibleNoteColumns").unwrap(),
            number_of_visible_effect_columns: find_tag_text_parsed!(n, "NumberOfVisibleEffectColumns").unwrap(),
            volume_column_is_visible: find_tag_text_parsed!(n, "VolumeColumnIsVisible").unwrap(),
            panning_column_is_visible: find_tag_text_parsed!(n, "PanningColumnIsVisible").unwrap(),
            delay_column_is_visible: find_tag_text_parsed!(n, "DelayColumnIsVisible").unwrap()
        }
    ).collect()
}

fn make_pattern_pool(doc: &roxmltree::Document) -> PatternPool {
    let pattern_pool = find_tag!(doc.root_element(), "PatternPool").unwrap();

    PatternPool {
        patterns: collect_patterns(&pattern_pool)
    }
}

fn collect_patterns(pattern_pool: &roxmltree::Node) -> Vec<Pattern> {
    let patterns = find_tag!(pattern_pool, "Patterns").unwrap();

    patterns.children().filter(|n| n.is_element()).map(|n|
        Pattern {
            number_of_lines: find_tag_text_parsed!(n, "NumberOfLines").unwrap(),
            tracks: collect_pattern_tracks(&n)
        }
    ).collect()
}

fn collect_pattern_tracks(pattern: &roxmltree::Node) -> Vec<PatternTrack> {
    let tracks = find_tag!(pattern, "Tracks").unwrap();

    tracks.children().filter(|n| n.is_element()).map(|n|
        PatternTrack {
            r#type: n.attribute("type").unwrap().to_string(),
            alias_pattern_index: find_tag_text_parsed!(n, "AliasPatternIndex").unwrap(),
            lines: collect_lines(&n)
        }
    ).collect()
}

fn collect_lines(pattern_track: &roxmltree::Node) -> Vec<Line> {
    match find_tag!(pattern_track, "Lines") {
        Some(lines) => {
            lines.children().filter(|n| n.is_element()).map(|n|
                Line {
                    index: n.attribute("index").unwrap().parse().unwrap(),
                    note_columns: collect_note_columns(&n),
                    effect_columns: collect_effect_columns(&n)
                }
            ).collect()
        },
        None => { vec![] }
    }
}

fn collect_note_columns(lines: &roxmltree::Node) -> Vec<Option<NoteColumn>> {
    match find_tag!(lines, "NoteColumns") {
        Some(note_columns) => {
            note_columns.children().filter(|n| n.is_element() && n.has_tag_name("NoteColumn")).map(|n|
                if n.has_children() {
                    Some(NoteColumn {
                        note: find_tag_text!(n, "Note", "   ").unwrap().to_string(),
                        instrument: find_tag_text!(n, "Instrument", "..").unwrap().to_string(),
                        volume: find_tag_text!(n, "Volume", "..").unwrap().to_string(),
                        panning: find_tag_text!(n, "Panning", "..").unwrap().to_string()
                    })
                } else {
                    None
                }
            ).collect()
        },
        None => { vec![] }
    }
}

fn collect_effect_columns(lines: &roxmltree::Node) -> Vec<Option<EffectColumn>> {
    match find_tag!(lines, "EffectColumns") {
        Some(effect_columns) => {
            effect_columns.children().filter(|n| n.is_element() && n.has_tag_name("EffectColumn")).map(|n|
                if n.has_children() {
                    Some(EffectColumn {
                        value: find_tag_text!(n, "Value", "00").unwrap().to_string(),
                        number: find_tag_text!(n, "Number", "  ").unwrap().to_string(),
                    })
                } else {
                    None
                }
            ).collect()
        },
        None => { vec![] }
    }
}

fn make_pattern_sequence(doc: &roxmltree::Document) -> PatternSequence {
    let pattern_sequence = find_tag!(doc.root_element(), "PatternSequence").unwrap();

    PatternSequence {
        sequence_entries: collect_sequence_entries(&pattern_sequence)
    }
}

fn collect_sequence_entries(pattern_sequence: &roxmltree::Node) -> Vec<SequenceEntry> {
    match find_tag!(pattern_sequence, "SequenceEntries") {
        Some(sequence_entry) => {
            sequence_entry.children().filter(|n| n.is_element() && n.has_tag_name("SequenceEntry") && n.has_children()).map(|n|
                SequenceEntry {
                    pattern: find_tag_text_parsed!(n, "Pattern").unwrap(),
                    section_name: find_tag_text_parsed!(n, "SectionName").unwrap(),
                    muted_tracks: collect_muted_tracks(&n)
                }
            ).collect()
        },
        None => { vec![] }
    }
}

fn collect_muted_tracks(sequence_entry: &roxmltree::Node) -> Vec<u32> {
    match find_tag!(sequence_entry, "MutedTracks") {
        Some(muted_tracks) => {
            muted_tracks.children()
                .filter(|n| n.is_element() && n.has_tag_name("MutedTrack"))
                .map(|n| n.text().unwrap().parse().unwrap())
                .collect()
        },
        None => { vec![] }
    }
}
