#[macro_use]
extern crate simple_error;

mod renoise;
mod pattern_font;
mod renderer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("usage: {} FILENAME", args[0]);
        return;
    }

    let filename = std::path::Path::new(&args[1]);
    println!("Reading {}", filename.display());
    let song = renoise::Song::from_xrns(&filename).unwrap();

    println!("Loaded song {} by {}", song.global_song_data.song_name, song.global_song_data.artist);

    println!("Rendering images");
    for i in 0..song.pattern_pool.patterns.len() {
        println!("pattern {:02}", i);
        let image = renderer::render(&song, i);
        println!("writing file...");
        image.save(format!("pattern{:02}.png", i)).unwrap();
    }
}
