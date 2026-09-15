# jellycard

Generates Jellyfin library card artwork: bold rounded text in a purple-to-cyan
horizontal gradient on a transparent background.

<img width="200" alt="live_tv" src="https://github.com/user-attachments/assets/1a16f7f9-18cf-400f-9647-9a938a6ccc70" />
<img width="200" alt="trailers" src="https://github.com/user-attachments/assets/545e53fd-372c-413d-aafc-d11fb0bd96d1" />
<img width="200" alt="videos" src="https://github.com/user-attachments/assets/8e28eae9-9f1d-42af-8ea8-2846562396c2" />

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
brew install otaviocc/apps/jellycard
cargo install jellycard --locked
```

Or take a binary from the [latest
release](https://github.com/otaviocc/jellycard/releases/latest): Linux x86-64
and ARM64, a macOS universal binary, and Windows x86-64.

## License

MIT — see `LICENSE`. `assets/Fredoka.ttf` is licensed separately, under the SIL
Open Font License 1.1; see `assets/OFL.txt`.
