# docs/[radios](../radios.md)/alinco_djmd5t

`alinco_djmd5t` allows you to parse and generate CSV exports from the Alinco DJ_MD5 CPS.

This driver was targeted and tested on a DJ-MD5TGP running the v1.13e firmware and using the v1.13e CPS. The CPS, firmware, drivers, and manuals are [here](https://www.remtronix.com/digital-radio/discontinued/dj-md5tgp/). It should work with a DJ-MD5T, as the only difference appears to be GPS and digital APRS on the TGP variant.

## Properties/Specifications

Based on v1.13e CPS and [specifications](https://www.alinco.com/Products/dmr/DJ-MD5/DJ-MD5.pdf)

* TX Frequencies: 136 - 174 MHz, 400 - 480 MHz
* Supported modes: FM, NFM, DMR
* Max channels: 4000
* Max channel name length: 16
* Max zones: 250
* Max zone name length: 16
* Max channels per zone: 250
* Max scanlists: 250
* Max scanlist name length: 16
* Max channels per scanlist: ??
* Max talkgroups: 10000
* Max talkgroup name length: 16
* Max talkgroup lists: 250
* Max talkgroups per talkgroup list: ??
* Max DMR IDs: 250
* Max DMR ID name length: 16

## CPS export notes

The CPS exports a set of CSV files with accompanying *.LST (Tool > Export > Export All(Default CSV FileName)).

Some of the fields are not particularly well-named.

In general, the format is very similar to the Anytone CPS exports, and much of the same parsing logic can be used.

### Transmit Power

The CPS options are `[Small, Low, Mid, High]` but the CSV uses `[Low, Mid, High, Turbo]` respectively. Presumably these map to `[0.2W, 1.0W, 2.5W, 5.0W]` as those are the power levels in the specifications, but this needs to be tested with a power meter.

### Busy Lock/TX Permit

For analog channels, the CPS calls this **Busy Lock** and the options are `[Off, Repeater, Busy]`. For digital channels, the CPS calls this **TX Permit** and the options are `[Always, ChannelFree, Different Color Code, Same Color Code]`.

In the CSV, the column is called "Busy Lock/TX Permit" and has a union of the above options. The default for analog channels is `Off` and the default for digital channels is `Always`.

The manual describes Busy Lock as follows:

> 16. Busy Lock<br>
> **Always**: Always allows transmissions<br>
> **Repeater**: Will not allow transmit when receiving matched carrier but unmatched
CTCSS/DCS<br>
> **Busy**: Will not allow transmit when receiving matched carrier<br>

The manual calls TX Permit "TX Allow" instead:

> 10. TX Allow<br>
> **Always**: Always allow to transmit<br>
> **Different CC**: Allow to transmit when radio receives a matching carrier signal but
different color code<br>
> **Same CC**: Allow to transmit when radio receives a matching carrier signal and it has same color code<br>
