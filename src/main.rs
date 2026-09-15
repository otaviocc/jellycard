// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Otávio Cordeiro

//! Command line entry point: names in, Jellyfin library card PNGs out.

mod render;

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
usage:
    jellycard \"4K Movies\"    write ./4k_movies.png
    jellycard -              read names from stdin, one card per line";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let names = match args.as_slice() {
        [arg] if arg == "-" => match read_stdin() {
            Ok(names) => names,
            Err(error) => return fail(&format!("reading stdin: {error}")),
        },
        [arg] if !arg.starts_with('-') => vec![arg.clone()],
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    if names.is_empty() {
        return fail("no names given");
    }

    for name in &names {
        match write_card(name) {
            Ok(path) => println!("{}", path.display()),
            Err(error) => return fail(&error),
        }
    }
    ExitCode::SUCCESS
}

fn read_stdin() -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let name = line.trim();
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

fn write_card(name: &str) -> Result<PathBuf, String> {
    let slug = slug(name).ok_or_else(|| format!("{name:?} has no usable filename"))?;
    let rgba = render::render(name).map_err(|error| error.to_string())?;
    let path = PathBuf::from(format!("{slug}.png"));
    write_png(&path, &rgba).map_err(|error| format!("writing {}: {error}", path.display()))?;
    Ok(path)
}

fn slug(name: &str) -> Option<String> {
    let mut slug = String::new();
    for ch in name.trim().chars() {
        if ch.is_whitespace() {
            if !slug.is_empty() && !slug.ends_with('_') {
                slug.push('_');
            }
        } else if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            slug.extend(ch.to_lowercase());
        }
    }
    let slug = slug.trim_matches('_').to_string();
    (!slug.is_empty()).then_some(slug)
}

fn write_png(path: &Path, rgba: &[u8]) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, render::WIDTH, render::HEIGHT);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(rgba)?;
    Ok(())
}

fn fail(message: &str) -> ExitCode {
    let _ = writeln!(std::io::stderr(), "jellycard: {message}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_library_names() {
        assert_eq!(slug("4K Movies").as_deref(), Some("4k_movies"));
        assert_eq!(slug("Movies").as_deref(), Some("movies"));
        assert_eq!(slug("  Home   Videos  ").as_deref(), Some("home_videos"));
        assert_eq!(slug("Kids' Shows!").as_deref(), Some("kids_shows"));
        assert_eq!(slug("!!!"), None);
        assert_eq!(slug(""), None);
    }
}
