# plungle

`plungle` is a command-line tool for radio codeplug conversion. It is designed to take a CSV or similar export from one CPS and convert it to a device-agnostic format, which can then be converted back to a CSV or similar format for import by another CPS, thus allowing you to translate a codeplug for one radio to another. It relies on the CPS to import/export and thus does not reverse-engineer the actual codeplug format. This means the process has a lot of steps, but also it probably won't brick your radio(s). However, you still need the CPS for _both_ radios.

`plungle` also performs basic validation on the codeplug, providing rudimentary detection of some data-entry errors or source data issues.

Please also see [qdmr](https://dm3mat.darc.de/qdmr/), a tool by DM3MAT that reverse-engineers the codeplug formats of various radios and also provides a full UI and programming functionality. This may be easier to use if your radio is supported.

> [!WARNING]
> This tool is provided without warranty, and the user assumes all risks. You must verify the resulting codeplug yourself, and ensure that you are transmitting within your license privileges.

## Installation

## Usage

Read a CSV export of a codeplug:
```
plungle read <radio-a> <csv-export-dir> > codeplug.json
```

Write a codeplug CSV export for a different radio:
```
plungle write <radio-b> codeplug.json <output-dir>
```

### Example

For this example, we are going to convert a codeplug for the Retevis RT3S running OpenGD77 to a codeplug for the Anytone AT-D878UV.

First, you must open the codeplug in the OpenGD77 CPS and export to CSV. For this example, we will assume this export directory is named `opengd77_csv`.

Read the exported codeplug (plungle uses JSON as an intermediary data format):

```
plungle read opengd77_rt3s opengd77_csv > codeplug.json
```

Write the codeplug export files for the target radio (where `output` is a directory that will be created containing CSV files to be imported into the Anytone CPS):

```
plungle write anytone_x78 codeplug.json output
```

## Documentation

Complete documentation is not yet written.

## Status/Issues

### Supported Radios

* Anytone D878UV (other Dx78 radios are untested but may work)
* Retevis RT3S running OpenGD77

## Future Plans

Future features that may eventually be added include:

* TOML, YAML, and CSV as intermediary data formats
* Generic Chirp input/output
* Merging multiple codeplugs
* Filtering codeplugs
* Batch editing operations
* Support for Motorola XPR 7550
* Support for Retevis RT3S on stock firmware, and TYT MD-UV380/MD-UV390 on both stock and OpenGD77 firmware
* Support for Ailunce HD1
* Support for Yaesu FT-3D
