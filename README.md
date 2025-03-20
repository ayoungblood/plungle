# plungle

plungle is a command-line tool for radio codeplug conversion. It is designed to take a CSV or similar export from one CPS and convert it to a device-agnostic format, which can then be converted back to a CSV or similar format for import by another CPS, thus allowing you to translate a codeplug for one radio to another. It relies on the CPS to import/export and thus does not reverse-engineer the actual codeplug format. This means the process has a lot of steps, but also it probably won't brick your radio(s). However, you still need the CPS for _both_ radios.

plungle also performs basic validation on the codeplug, providing rudimentary detection of some data-entry errors or source data issues.

Please also see [qdmr](https://dm3mat.darc.de/qdmr/), a tool by DM3MAT that reverse-engineers the codeplug formats of various radios and also provides a full UI and programming functionality. This may be easier to use if your radio is supported.

> [!CAUTION]
> This tool is provided without warranty, and the user assumes all risks. You must verify the resulting codeplug yourself, and ensure that you are transmitting within your license privileges.

> [!WARNING]
> plungle is still in its infancy. Expect breaking changes, incomplete output, and errors in reading/writing radio-specific files. Please report any issues.

## Installation

Currently, there are no compiled binaries available.

### Building from source

First, you need to [install Rust](https://www.rust-lang.org/tools/install). On *nix/MacOS, this is very straightforward. On Windows, it is highly recommended to use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install).

Then, clone the repository and build the project:

```
git clone https://github.com/ayoungblood/plungle.git
cd plungle
cargo build --release
```

The compiled binary will be located at `target/release/plungle`. You can add it to your path or install it system-wide:

```
# Option 1: Install to ~/.cargo/bin (included in PATH if you installed Rust via rustup)
cargo install --path .

# Option 2: Copy the binary to a location in your PATH
sudo cp target/release/plungle /usr/local/bin/
```

Verify the installation:

```
plungle --version
```

## Usage

Parse a codeplug export from Radio A into an intermediary format
```
plungle parse <radio-a> <csv-export-dir> codeplug.json
```

Generate a codeplug export for Radio B from an intermediary format
```
plungle generate <radio-b> codeplug.json <output-dir>
```

Merge three codeplugs into one codeplug
```
plungle merge codeplug-1.json codeplug-2.json codeplug-3.json --format=json > output.json
```
The merge argument syntax isn't great. It will be improved.

### Example

For this example, we are going to convert a codeplug for the Retevis RT3S running OpenGD77 to a codeplug for the Anytone AT-D878UV.

First, you must open the codeplug in the OpenGD77 CPS and export to CSV. For this example, we will assume this export directory is named `opengd77_csv`.

Parse the exported codeplug (plungle uses JSON as an intermediary data format):

```
plungle parse opengd77_rt3s opengd77_csv codeplug.json
```

Generate the codeplug export files for the target radio (where `output` is a directory that will be created containing CSV files to be imported into the Anytone CPS):

```
plungle generate anytone_x78 codeplug.json output
```

## Documentation

Complete documentation is not yet written.

## Status/Issues

### Supported Radios

* Anytone D878UV (other Dx78 radios are untested but may work)
* Retevis RT3S running OpenGD77 (any other OpenGD77 radio should be supported, but has not been tested)
* Alinco DJ-MD5TGP (other DJ-MD5x radios are untested but may work)
* Ailunce HD1 (support is very poor due to limitations of the HD1 CPS)
* TYT MD-UV390 (support is very poor due to limitations of the MD-UV390 CPS)
* Generic [CHIRP](https://chirpmyradio.com/projects/chirp/wiki/Home) support

## Future Plans

Future features that may eventually be added include:

* CSV as an intermediary data format
* [qdmr](https://github.com/hmatuschek/qdmr)-compatible import/export
* Filtering codeplugs
* Batch editing operations
* Support for Motorola XPR 7550/e (VHF/UHF)
* Support for Motorola XPR 6550 (VHF/UHF)
* Support for Retevis RT3S on stock firmware, and TYT MD-UV380/MD-UV390 on OpenGD77 firmware
* Support for Yaesu FT-3D
* Support for Radioddity DB25-D
* Improved support for scanlists

## Contributing

First, thank you for your interest in this project.

If you wish to contribute to this project, please first discuss the change you wish to make via Github issue, email, or other method with the owner of this project.

This project was developed primarily for personal use, and also as a way to learn Rust. The author is not a software engineer, just a hardware person stumbling along through a new language. As the sole maintainer of this project, I cannot accept contributions that I cannot understand. Please keep that in mind, lest you decide to refactor the whole project because it's "bad".
