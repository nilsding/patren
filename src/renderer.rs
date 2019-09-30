extern crate image;

struct ColorPair {
    normal: image::Rgba<u8>,
    highlighted: image::Rgba<u8>
}

impl ColorPair {
    fn get(&self, highlighted: bool) -> image::Rgba<u8> {
        if highlighted { self.highlighted }
        else { self.normal }
    }
}

static COLOR_BACK:      ColorPair = ColorPair { normal: image::Rgba([0x15, 0x15, 0x15, 255]), highlighted: image::Rgba([0x29, 0x29, 0x29, 255]) };
static COLOR_DEFAULT:   ColorPair = ColorPair { normal: image::Rgba([0x94, 0x94, 0x94, 255]), highlighted: image::Rgba([0xFF, 0xFF, 0xFF, 255]) };
static COLOR_VOLUME:    ColorPair = ColorPair { normal: image::Rgba([0xD4, 0xCE, 0x2A, 255]), highlighted: image::Rgba([0xBF, 0xAE, 0x25, 255]) };
static COLOR_PANNING:   ColorPair = ColorPair { normal: image::Rgba([0x9D, 0xD6, 0x8C, 255]), highlighted: image::Rgba([0x81, 0xAF, 0x72, 255]) };
static COLOR_PITCH:     ColorPair = ColorPair { normal: image::Rgba([0xB4, 0x4F, 0x21, 255]), highlighted: image::Rgba([0x9B, 0x44, 0x1D, 255]) };
static COLOR_DELAY:     ColorPair = ColorPair { normal: image::Rgba([0x42, 0xC1, 0xEA, 255]), highlighted: image::Rgba([0x3D, 0xB4, 0xDA, 255]) };
static COLOR_GLOBAL_FX: ColorPair = ColorPair { normal: image::Rgba([0xFD, 0x97, 0x14, 255]), highlighted: image::Rgba([0xC6, 0x76, 0x10, 255]) };
static COLOR_OTHER_FX:  ColorPair = ColorPair { normal: image::Rgba([0xBA, 0x68, 0xBB, 255]), highlighted: image::Rgba([0x9A, 0x56, 0x9B, 255]) };
static COLOR_DSP_FX:    ColorPair = ColorPair { normal: image::Rgba([0xDB, 0xDB, 0xDB, 255]), highlighted: image::Rgba([0xE5, 0xE5, 0xE5, 255]) };
static COLOR_UNUSED_FX: ColorPair = ColorPair { normal: image::Rgba([0x9C, 0x9C, 0x9C, 255]), highlighted: image::Rgba([0x9C, 0x9C, 0x9C, 255]) };

const CHAR_WIDTH:         u32 = super::pattern_font::CHAR_WIDTH as u32;
const TRACK_SPACING_X:    u32 = 6;
const TRACK_SPACING_X_FX: u32 = 3;
const TRACK_WIDTH_NOTE:   u32 = TRACK_SPACING_X_FX + 5 * CHAR_WIDTH; // e.g. C-500
const TRACK_WIDTH_VOL:    u32 = TRACK_SPACING_X_FX + 2 * CHAR_WIDTH; // e.g. 7F
const TRACK_WIDTH_FX:     u32 = TRACK_SPACING_X_FX + 4 * CHAR_WIDTH; // e.g. ZT04

const TRACK_SPACING_Y: u32 = 2;

pub fn render(song: &super::renoise::Song, pattern: usize) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let width: u32 = 2 + x_offset_upto_track(&song, song.tracks.len());
    let height: u32 = 2 + song.pattern_pool.patterns[pattern].number_of_lines * (CHAR_WIDTH + TRACK_SPACING_Y);

    println!("image size: {}x{}", width, height);
    let mut imgbuf = image::ImageBuffer::new(width, height);

    render_pattern(&mut imgbuf, &song, pattern);

    imgbuf
}

fn x_offset_for_track(song: &super::renoise::Song, track_index: usize) -> u32 {
    let mut offset: u32 = 0;

    let track = &song.tracks[track_index];
    offset += track.number_of_visible_note_columns * TRACK_WIDTH_NOTE;
    if track.volume_column_is_visible {
        offset += track.number_of_visible_note_columns * TRACK_WIDTH_VOL
    }
    if track.panning_column_is_visible {
        offset += track.number_of_visible_note_columns * TRACK_WIDTH_VOL
    }
    offset += track.number_of_visible_effect_columns * TRACK_WIDTH_FX;
    offset += TRACK_SPACING_X;

    offset
}

fn x_offset_upto_track(song: &super::renoise::Song, track_index: usize) -> u32 {
    (0..track_index).map(|i| x_offset_for_track(&song, i)).sum()
}

fn render_pattern(mut imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, song: &super::renoise::Song, pattern: usize) {
    let pat = &song.pattern_pool.patterns[pattern];
    for (index, track) in pat.tracks.iter().enumerate() {
        let x: u32 = x_offset_upto_track(&song, index);
        let track_info = &song.tracks[index];
        let lines: &Vec<super::renoise::Line> = if track.alias_pattern_index < 0 {
            &track.lines
        } else {
            &song.pattern_pool.patterns[track.alias_pattern_index as usize].tracks[index].lines
        };

        let mut rendered_lines: Vec<u32> = vec![];

        // // render highlighted background
        // for line in (0..pat.number_of_lines).step_by(song.global_song_data.lines_per_beat as usize) {
        //     for x in 0..imgbuf.width() {
        //         for y in 0..CHAR_WIDTH {
        //             let y_offset: u32 = line * (CHAR_WIDTH + TRACK_SPACING_Y);
        //             imgbuf.put_pixel(x, y + y_offset, COLOR_BACK.highlighted);
        //         }
        //     }
        // }

        // render lines in track
        for line in lines.iter() {
            if line.index >= pat.number_of_lines {
                continue;
            }
            rendered_lines.push(line.index);

            let y: u32 = line.index * (CHAR_WIDTH + TRACK_SPACING_Y);
            let mut x_offset: u32 = 0;
            let highlighted: bool = line.index % &song.global_song_data.lines_per_beat == 0;

            for (i_note, note_column) in line.note_columns.iter().enumerate() {
                if i_note as u32 >= track_info.number_of_visible_note_columns {
                    break;
                }
                x_offset = render_note_column(&mut imgbuf, &note_column, &track_info, highlighted, x, x_offset, y);
            }

            if line.note_columns.len() < track_info.number_of_visible_note_columns as usize {
                for _ in line.note_columns.len()..track_info.number_of_visible_note_columns as usize {
                    x_offset = render_note_column(&mut imgbuf, &None, &track_info, highlighted, x, x_offset, y);
                }
            }

            for (i_fx, effect_column) in line.effect_columns.iter().enumerate() {
                if i_fx as u32 >= track_info.number_of_visible_effect_columns {
                    break;
                }

                x_offset = render_effect_column(&mut imgbuf, &effect_column, highlighted, x, x_offset, y);
            }

            if line.effect_columns.len() < track_info.number_of_visible_effect_columns as usize {
                for _ in line.effect_columns.len()..track_info.number_of_visible_effect_columns as usize {
                    x_offset = render_effect_column(&mut imgbuf, &None, highlighted, x, x_offset, y);
                }
            }
        }

        // render empty strings
        for line in 0..pat.number_of_lines {
            if rendered_lines.contains(&line) {
                continue;
            }

            let y: u32 = line as u32 * (CHAR_WIDTH + TRACK_SPACING_Y);
            let mut x_offset: u32 = 0;
            let highlighted: bool = line % &song.global_song_data.lines_per_beat == 0;

            for _ in 0..track_info.number_of_visible_note_columns {
                x_offset = render_note_column(&mut imgbuf, &None, &track_info, highlighted, x, x_offset, y);
            }
            for _ in 0..track_info.number_of_visible_effect_columns {
                x_offset = render_effect_column(&mut imgbuf, &None, highlighted, x, x_offset, y);
            }
        }
    }
}

fn render_note_column(mut imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, note_column: &Option<super::renoise::NoteColumn>, track_info: &super::renoise::Track, highlighted: bool, x: u32, x_offset: u32, y: u32) -> u32 {
    let empty_str_note = &String::from("   ");
    let empty_str_vol = &String::from("..");

    let mut x_offset = x_offset;
    match note_column {
        Some(note) => {
            render_text(&mut imgbuf, &note.note, x + x_offset, y, &COLOR_DEFAULT.get(highlighted));
            render_text(&mut imgbuf, &note.instrument, x + x_offset + (CHAR_WIDTH * 3), y, &COLOR_DEFAULT.get(highlighted));
            x_offset += TRACK_WIDTH_NOTE;

            if track_info.volume_column_is_visible {
                render_text(&mut imgbuf, &note.volume, x + x_offset, y, &COLOR_VOLUME.get(highlighted));
                x_offset += TRACK_WIDTH_VOL;
            }

            if track_info.panning_column_is_visible {
                render_text(&mut imgbuf, &note.panning, x + x_offset, y, &COLOR_PANNING.get(highlighted));
                x_offset += TRACK_WIDTH_VOL;
            }
        },
        None => {
            render_text(&mut imgbuf, &empty_str_note, x + x_offset, y, &COLOR_DEFAULT.get(highlighted));
            render_text(&mut imgbuf, &empty_str_vol, x + x_offset + (CHAR_WIDTH * 3), y, &COLOR_DEFAULT.get(highlighted));
            x_offset += TRACK_WIDTH_NOTE;

            if track_info.volume_column_is_visible {
                render_text(&mut imgbuf, &empty_str_vol, x + x_offset, y, &COLOR_VOLUME.get(highlighted));
                x_offset += TRACK_WIDTH_VOL;
            }

            if track_info.panning_column_is_visible {
                render_text(&mut imgbuf, &empty_str_vol, x + x_offset, y, &COLOR_PANNING.get(highlighted));
                x_offset += TRACK_WIDTH_VOL;
            }
        }
    }
    x_offset
}

fn render_effect_column(mut imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, effect_column: &Option<super::renoise::EffectColumn>, highlighted: bool, x: u32, x_offset: u32, y: u32) -> u32 {
    let empty_str_fx = &String::from("  ");

    let mut x_offset = x_offset;
    match effect_column {
        Some(effect) => {
            let color = fx_color(&effect.number);
            render_text(&mut imgbuf, &fx_command(&effect.number), x + x_offset, y, &color.get(highlighted));
            render_text(&mut imgbuf, &effect.value, x + x_offset + (CHAR_WIDTH * 2), y, &color.get(highlighted));
            x_offset += TRACK_WIDTH_FX;
        },
        None => {
            render_text(&mut imgbuf, &empty_str_fx, x + x_offset, y, &COLOR_DEFAULT.get(highlighted));
            render_text(&mut imgbuf, &empty_str_fx, x + x_offset + (CHAR_WIDTH * 2), y, &COLOR_DEFAULT.get(highlighted));
            x_offset += TRACK_WIDTH_FX;
        }
    }
    x_offset
}

fn render_text(mut imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, text: &String, x: u32, y: u32, color: &image::Rgba<u8>) {
    let rendered_chars = text.as_bytes().iter().map(|b| super::pattern_font::char(*b));

    for (index, ch) in rendered_chars.enumerate() {
        render_char(&mut imgbuf, ch, x + 8 * index as u32, y, &color);
    }
}

fn render_char(imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, ch: [u8; 8], x: u32, y: u32, color: &image::Rgba<u8>) {
    for (index, row) in ch.iter().enumerate() {
        let real_y: u32 = y + index as u32;

        let mut num = *row;
        let mut x_offset = 0;
        while num != 0 {
            if num >> 7 == 1 {
                imgbuf.put_pixel(x + x_offset, real_y, *color);
            }
            x_offset += 1;
            num <<= 1;
        }
    }
}

fn fx_color(number: &String) -> &ColorPair {
    match number.as_bytes() {
        [b'Z', b'T'] | [b'Z', b'L'] | [b'Z', b'K'] | [b'Z', b'G'] | [b'Z', b'B'] | [b'Z', b'D'] => &COLOR_GLOBAL_FX,
        [_, b'A'] | [_, b'U'] | [_, b'D'] | [_, b'G'] | [_, b'V'] => &COLOR_PITCH,
        [_, b'I'] | [_, b'O'] | [_, b'T'] | [_, b'C'] | [_, b'M'] | [_, b'L'] => &COLOR_VOLUME,
        [_, b'S'] | [_, b'B'] | [_, b'E'] | [_, b'Q'] | [_, b'R'] | [_, b'Y'] => &COLOR_DELAY,
        [_, b'N'] | [_, b'P'] | [_, b'W'] => &COLOR_PANNING,
        [_, b'X'] | [_, b'Z'] | [_, b'J'] => &COLOR_OTHER_FX,
        _ => &COLOR_UNUSED_FX,
    }
}

fn fx_command(number: &String) -> String {
    if number.as_bytes()[0] == b'0' {
        String::from_utf8(vec!(b' ', number.as_bytes()[1])).unwrap()
    } else {
        number.to_string()
    }
}
