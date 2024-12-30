# plungle

`plungle` is a command-line tool for radio codeplug conversion. It is designed to take a CSV or similar export from one CPS and convert it to a device-agnostic format, which can then be converted back to a CSV or similar format for import by another CPS, thus allowing you to translate a codeplug for one radio to another. It relies on the CPS to import/export and thus does not reverse-engineer the actual codeplug format. This means the process has a lot of steps, but also it probably won't brick your radio(s). However, you still need the CPS for _both_ radios.

Please also see [qdmr](https://dm3mat.darc.de/qdmr/), a tool by DM3MAT that reverse-engineers the codeplug formats of various radios and also provides a full UI and programming functionality. This may be easier to use if your radio is supported.

## Installation

## Usage

Read a CSV export of a codeplug

```
plungle read <radio-a> <export-dir> > codeplug.yaml
```

Validate the codeplug

```
plungle validate codeplug.yaml
```

Write a codeplug for a different radio

```
plungle write <radio-b> codeplug.yaml <output-dir>
```

### Example

For this example, we are going to convert an Anytone D878 codeplug to an OpenGD77 codeplug.

First, you must open the codeplug in the Anytone CPS and export to CSV (in this example, the export directory is called `anytone_csv`).

Read the exported codeplug (plungle uses YAML as an intermediary data format):

```
plungle read anytone-d878 anytone_csv > codeplug.yaml
```

Optionally, validate the parsed codeplug, this does some quick checks and warns if anything seems amiss.

```
plungle validate codeplug.yaml
```

Write the codeplug export files for the target radio (where `output` is a directory that will be  created containing CSV files to be imported into the OpenGD77 CPS):

```
plungle write opengd77-rt3s codeplug.yaml output
```

## Documentation

Complete documentation is not yet written.

## Issues
