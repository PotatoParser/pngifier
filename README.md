# pngifier

> **Quickly** & **easily** convert **any** file into a PNG and back!

*Google Photos == Unlimited Online File Storage :D*

![test](https://user-images.githubusercontent.com/45542237/88219546-9f2b8e00-cc16-11ea-8e2d-df97d60c7ab8.gif)

## Features

- **Supports Linux, MacOS, Windows**
- Supports streaming output
- Built-in PNG CRC verification
- Supports 4 color modes:
  - Greyscale
  - Greyscale + Alpha
  - RGB
  - RGB + Alpha
- Supports 8-bit & 16-bit depths
- Customizability
  - Adjust buffer sizes
  - Adjust width of image
  - Adjust height of image
- Displays progress bars

## Installation

> **Getting in on the action!**

[**Easiest Method**]

Download a pre-built binary from releases: **[Pngifier Releases](https://github.com/PotatoParser/pngifier/releases)**

   > **Note:** default backend is located within `/bin`

<br/>

Compiling source code:

1. Clone this repo
2. Enter the repository via `cd pngifier`
3. Compile source code `cargo build` or `cargo build --releases`
   
   > **Note:** `/configs` contains different `*.toml` files that change the DEFLATE/INFLATE backend

## Examples

> **The easy way to get started!**

To quickly convert a file into a png: `file.txt` -> `file.txt.png`<br/>
`pngifier encode file.txt`

To quickly convert a png back into a file: `file.txt.png` -> `file.txt`<br/>
`pngifier decode file.txt.png`

Encode to a different file: `file.txt -> file2.png`<br/>
`pngifier encode file.txt file2.png`

Decode to a different file: `file2.png -> file.txt`<br/>
`pngifier decode file2.png file.txt`

Encoding as a 16-bit, RGBA PNG:<br/>
`pngifier encode -t=rgba -b=16 file.txt`

Stream data<br/>
`pngifier encode file.txt --stream`

Example streaming<br/>
`pngifier encode file.txt --stream > 2>&1`



## CLI Usage

> **Everything!**

### Encoding:

```
pngifier-encode

USAGE:
    pngifier encode [FLAGS] [OPTIONS] <INPUT> [OUTPUT]

FLAGS:
    -y, --yes         Override all values with yes
    -p, --progress    Displays the progress
    -s, --silent      Prevents all outputs
        --stream      Streams the output to stdout
        --trim        Trims the output (removes trailing null bytes)
    -v, --verbose     Verbose output
        --verify      Verifies the file as a png before attempting to read it
        --help        Prints help information

OPTIONS:
    -b, --buffer <BYTES>       Sets the limiting buffer size (ie: 100, 1kb, 10mb, 1gb)
    -t, --type <COLOR_TYPE>    Sets the color type (0, 2, 4, 6, g, ga, rgb, rgba)
    -d, --depth <DEPTH>        Sets color depth. Bit depths of 8-bit and 16-bit are supported
    -h, --height <HEIGHT>      Sets the height of the image in pixels
    -w, --width <WIDTH>        Sets the width of the image in pixels

ARGS:
    <INPUT>     Sets the input file to use
    <OUTPUT>    Sets the output file
```

### Decoding:

```
pngifier-decode

USAGE:
    pngifier decode [FLAGS] [OPTIONS] <INPUT> [OUTPUT]

FLAGS:
    -y, --yes         Override all values with yes
    -p, --progress    Displays the progress
    -s, --silent      Prevents all outputs
        --stream      Streams the output to stdout
        --trim        Trims the output (removes trailing null bytes)
    -v, --verbose     Verbose output
        --verify      Verifies the file as a png before attempting to read it
    -h, --help        Prints help information

OPTIONS:
    -b, --buffer <BYTES>    Sets the limiting buffer size (ie: 100, 1kb, 10mb, 1gb)

ARGS:
    <INPUT>     Sets the input file to use
    <OUTPUT>    Sets the output file
```

## Benchmarks

> **How fast is fast?**

>CPU: i7 - 7700 HQ @ 2.8 GHz
>
>RAM: 16 GB 2400 MHz
>
>SSD: 2.8 GB/s read, 1.4 GB/s write
>
>OS: Windows 10 Home

### Encoding:

| File size                   | miniz.c [default] | flate2 (miniz_oxide) | cloudflare-zlib | zlib        |
| :-------------------------- | ----------------- | -------------------- | --------------- | ----------- |
| 1,084,735,488 bytes (~1 GB) | **9.1882012s**    | 10.9290874s          | 25.98657s       | 31.5629587s |
| 100,667,392 bytes (~100 MB) | **0.3018865s**    | 0.4424806s           | 0.3912431s      | 0.6884911s  |
| 11,223,040 bytes (~10 MB)   | **0.1293564s**    | 0.1475764s           | 0.2994075s      | 0.3356308s  |

### Decoding:

| File size                   | miniz.c [default] | flate2 (miniz_oxide) | cloudflare-zlib | zlib       |
| :-------------------------- | ----------------- | -------------------- | --------------- | ---------- |
| 1,084,735,488 bytes (~1 GB) | **4.2408762s**    | 4.571307s            | 4.7803391s      | 4.702615s  |
| 100,667,392 bytes (~100 MB) | **0.220.8307s**   | 0.2700087s           | 0.2546378s      | 0.2389715s |
| 11,223,040 bytes (~10 MB)   | **0.0573943s**    | 0.0626758s           | 0.0597192s      | 0.0597747s |