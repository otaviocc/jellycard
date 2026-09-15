# jellycard

Generates Jellyfin library card artwork: bold rounded text in a purple-to-cyan
horizontal gradient on a transparent background.

## Usage

```sh
jellycard "4K Movies"                  # writes ./4k_movies.png
echo "Anime" | jellycard -             # writes ./anime.png
printf 'Anime\nKids\n' | jellycard -   # one card per non-empty line
```

The file is written into the current directory, named after the library
(lowercased, spaces to underscores), and its path is printed to stdout.

## Install

```sh
cargo install --path .
```

## License

MIT — see `LICENSE`. `assets/Fredoka.ttf` is licensed separately, under the SIL
Open Font License 1.1; see `assets/OFL.txt`.
