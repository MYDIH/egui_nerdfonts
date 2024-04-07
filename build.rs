use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use downloader::Downloader;

const DOWNLOAD_URL: &str =
    "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.2.0/NerdFontsSymbolsOnly.zip";

fn main() {
    let fonts_path = Path::new("./fonts");
    let archive_path = fonts_path.join("_nerdfonts_regular.zip");
    let ttf_path = fonts_path.join("nerdfonts_regular.ttf");

    // Delete the temporary archive before download
    std::fs::remove_file(&archive_path).ok();

    // We download the specified font
    Downloader::builder()
        .build()
        .unwrap()
        .download(&[downloader::Download::new(DOWNLOAD_URL).file_name(&archive_path)])
        .unwrap();

    // Decompressing the TTF file
    let mut archive = zip::ZipArchive::new(File::open(&archive_path).unwrap()).unwrap();
    let mut file = archive.by_index(0).unwrap(); // single file archive
    let mut ttf_file = File::create(&ttf_path).unwrap();
    io::copy(&mut file, &mut ttf_file).unwrap();

    // Delete the temporary archive
    fs::remove_file(&archive_path).unwrap();

    // We parse the font to generate the glyph variables
    let font_data = fs::read(&ttf_path).unwrap();
    let face = ttf_parser::Face::parse(&font_data, 0)
        .expect("Something went wrong while parsing the downloaded TTF file");

    // We gather the glyphs
    let mut glyphs = vec![];
    if let Some(table) = face.tables().cmap {
        if let Some(unicode_table) = table
            .subtables
            .into_iter()
            .find(|&t| t.encoding_id == 4 && t.platform_id == ttf_parser::PlatformId::Unicode)
        {
            unicode_table.codepoints(|cp| {
                let glyph_id = unicode_table.glyph_index(cp);
                let glyph_name = glyph_id.and_then(|id| {
                    face.glyph_name(id).map(|n| {
                        n.to_uppercase()
                            .replace("-", "_")
                            .replace(" ", "_")
                            .replace("#", "_")
                            .replace(".", "_")
                            .replace("/", "_")
                            .replace("(", "")
                            .replace(")", "")
                            .replace("!", "")
                    })
                });
                if let Some(name) = glyph_name {
                    glyphs.push((name, format!("\\u{{{:X}}}", cp)));
                }
            })
        }
    }

    // We sort the glyphs alphabetically
    glyphs.sort_by(|a, b| a.0.cmp(&b.0));

    // We create the output file
    let mut regular_variant_file = File::create(&Path::new("./src/variants/regular.rs")).unwrap();

    // We prepend the file with #![allow(unused)]
    regular_variant_file
        .write(b"#![allow(unused)]\n\n")
        .unwrap();

    let mut previous_name = ""; // Used to find duplicates
    let mut duplicate_counter = 0;
    for (name, codepoint) in glyphs.iter() {
        let sanitized_name = if name.chars().next().unwrap().is_numeric() {
            duplicate_counter = 0;
            format!("_{}", name)
        } else if previous_name == name {
            duplicate_counter += 1;
            format!("{}_V{}", name, duplicate_counter)
        } else {
            duplicate_counter = 0;
            name.to_owned()
        };

        write!(
            regular_variant_file,
            "pub const {}: &str = \"{}\";\n",
            sanitized_name, codepoint
        )
        .unwrap();

        previous_name = name;
    }

    println!("cargo::rerun-if-changed=build.rs");
}
