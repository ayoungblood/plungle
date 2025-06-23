# docs/[radios](../radios.md)/qdmr_generic

`qdmr_generic` supports import and export of [qdmr](https://github.com/hmatuschek/qdmr) codeplugs. qdmr is an excellent open-source tool that supports a number of radios from Radioddity, TYT, Anytone, and BTECH. See [qdmr: Supported Radios](https://dm3mat.darc.de/qdmr/#dev) for a full list with details on each radio's supported features.

## Properties/Specifications

## Codeplug format notes

qdmr uses YAML for codeplug serialization. Unfortunately there is not a safe and stable rust crate for YAML serialization/deserialization, so we to use [saphyr](https://docs.rs/saphyr/latest/saphyr/) to read and write YAML files.

### Admit

Per [qdmr documentation](https://dm3mat.darc.de/qdmr/manual/ch03s05.html), analog channels have the following options for the 'admit' field: [`Always`, `Free`, `Tone`], and digital channels have the following options for the 'admit' field: [`Always`, `Free`, `ColorCode`].
