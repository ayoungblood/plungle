# plungle

plungle is a command-line tool for radio codeplug conversion. It is designed to take a CSV or similar export from one CPS and convert it to a device-agnostic format, which can then be converted back to a CSV or similar format for import by another CPS, thus allowing you to translate a codeplug for one radio to another. It relies on the CPS to import/export and thus does not reverse-engineer the actual codeplug format. This means the process has a lot of steps, but also it probably won't brick your radio(s). However, you still need the CPS for _both_ radios.

plungle also performs basic validation on the codeplug, providing rudimentary detection of some data-entry errors or source data issues.

Please also see [qdmr](https://dm3mat.darc.de/qdmr/), a tool by DM3MAT that reverse-engineers the codeplug formats of various radios and also provides a full UI and programming functionality. This may be easier to use if your radio is supported.

> [!CAUTION]
> This tool is provided without warranty, and the user assumes all risks. You must verify the resulting codeplug yourself, and ensure that you are transmitting within your license privileges.

> [!WARNING]
> plungle is still in its infancy. Expect breaking changes, incomplete output, and errors in reading/writing radio-specific files. Please report any issues.

## 1. Installation

Currently, there are no compiled binaries available.

### 1.1 Building from source

#### 1.1.1 Install Rust

[Follow instructions for your platform](https://www.rust-lang.org/tools/install).

On *nix/MacOS, this is very straightforward. On Windows, it is highly recommended to use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install).

#### 1.1.2 Clone and build plungle

Clone the repository and build the project:

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

## 2. Usage

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

### 2.1 Example

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

## 3. Documentation

Complete documentation is not yet written.

## 4. Status/Issues

### 4.1 Supported Radios

plungle supports different radios (or, more accurately, different codeplug or CPS export formats) by using different "drivers" that convert to/from the vendor-specific format to a common data structure used by plungle. This data structure is easily serialized to JSON for storage, and plungle does its best to convert to other vendor-specific formats.

Support for different formats is managed by different drivers specific to radio models or software. For more information, see the linked documentation for each model.

| model | supported radios | status | notes |
|-------|------------------|--------|-------|
| [ailunce_hd1](docs/radios/ailunce_hd1.md) | Ailunce HD1 | 🟧 | very limited to CPS limitations |
| [alinco_djmd5t](docs/radios/alinco_djmd5t.md) | Alinco DJ-MD5T<br>Alinco DJ-MD5TGP | 🟩 | tested with DJ-MD5TGP<br>may work with other similar Alinco radios |
| [anytone_x78](docs/radios/anytone_x78.md) | Anytone D878UV<br>??? | 🟩 | tested with D878UV Plus modded for APRS<br>may work with similar Anytone radios |
| [chirp_generic](docs/radios/chirp_generic.md) | see [CHIRP](https://chirpmyradio.com/projects/chirp/wiki/Home#Supported-Radio-Models) | 🟨 | currently untested<br>should work with any FM radios supported by CHIRP |
| [opengd77_rt3s](docs/radios/opengd77_rt3s.md) | Retevis RT3S w/ [OpenGD77](https://www.opengd77.com/) | 🟩 | tested with RT3S on OpenGD77 20240908..<br>may work on other radios supported by OpenGD77 but needs additional testing |
| [tyt_mduv390](docs/radios/tyt_mduv390.md) | TYT MD-UV390 | 🟧 | very limited to due CPS limitations<br>may be removed as [qdmr](https://dm3mat.darc.de/qdmr/) support is planned |


## 5. Roadmap

### 5.1 Planned

Planned features/support:

* Support for [qdmr](https://dm3mat.darc.de/qdmr/), which supports numerous radios from Radioddity, TYT, Retevis, Anytone, and Baofeng
* Filtering/merging of codeplugs
* Support for Motorola XPR7550/XPR7550e
* Support for [CPEditor by David MM7DBT](https://www.cpeditor.co.uk/), which supports:
    - Kydera CDR-300UV
    - Retevis RT73
    - Radioddity DB25-D
    - Radioddity DB40-D
    - Radioddity GD-8
* Improved support for scanlists

### 5.2 Possible

Future features that may eventually be added include:

* Batch editing operations
* Support for Motorola XPR6550
* Support for Yaesu FT-3D
* TUI for editing codeplugs

If there is a radio or editor that you would like to see supported, please let me know.

## 6. Contributing

First, thank you for your interest in this project.

If you wish to contribute to this project, please first discuss the change you wish to make via Github issue, email, or other method with the owner of this project.

This project was developed primarily for personal use, and also as a way to learn Rust. The author is not a software engineer, just a hardware person stumbling along through a new language. As the sole maintainer of this project, I cannot accept contributions that I cannot understand. Please keep that in mind, lest you submit a PR refactoring the entire project because the code is ugly!
