# plungle

`plungle` is a command-line tool for radio codeplug conversion. It is designed to take a CSV or similar export from one CPS and convert it to a device-agnostic format, which can then be converted back to a CSV or similar format for import by another CPS, thus allowing you to translate a codeplug for one radio to another. It relies on the CPS to import/export and thus does not reverse-engineer the actual codeplug format. This means the process has a lot of steps, but also it probably won't brick your radio(s). However, you still need the CPS for _both_ radios.

Please also see [qdmr](https://dm3mat.darc.de/qdmr/), a tool by DM3MAT that reverse-engineers the codeplug formats of various radios and also provides a full UI and programming functionality. This may be easier to use if your radio is supported.

## Installation

## Usage

Read a CSV export of a codeplug

```
plungle read <radio-a> <csv-export-dir> > codeplug.json
```

Validate the codeplug

```
plungle validate codeplug.json
```

Write a codeplug for a different radio

```
plungle write <radio-b> codeplug.json <output-dir>
```

### Example

For this example, we are going to convert an Anytone D878 codeplug to an OpenGD77 codeplug.

First, you must open the codeplug in the Anytone CPS and export to CSV (in this example, the export directory is called `anytone_csv`).

Read the exported codeplug (plungle uses JSON as an intermediary data format):

```
plungle read anytone-d878 anytone_csv > codeplug.json
```

Optionally, validate the parsed codeplug, this does some quick checks and warns if anything seems amiss.

```
plungle validate codeplug.json
```

Write the codeplug export files for the target radio (where `output` is a directory that will be  created containing CSV files to be imported into the OpenGD77 CPS):

```
plungle write opengd77-rt3s codeplug.json output
```

## Documentation

Complete documentation is not yet written.

## Issues

## Future Plans

* Support JSON, TOML, YAML, and CSV as intermediary data formats
* Support Chirp input/output
* Support merging multiple codeplugs
