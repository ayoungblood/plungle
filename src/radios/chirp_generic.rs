// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;

// CSV Export Format
// CHIRP next-20241108
// CHIRP exports a single CSV file:
// - Location: channel index
// - Name: channel name
// - Frequency: frequency in MHz
// - Duplex: [+, -, (blank)]
// - Offset: offset in MHz, typ [0, 0.6, 5]
// - Tone: complicated
//     Tone -
// - rToneFreq: RX CTCSS frequency in Hz, 88.5 default
// - cToneFreq: TX(?) CTCSS frequency in Hz, 88.5 default
// - DtcsCode: DCS code, 23 default
// - DtcsPolarity: DCS polarity, NN default
// - RxDtcsCode: RX DCS code, 23 default
// - CrossMode: [Tone-Tone, ??]
// - Mode: [FM, NFM, ??]
// - TStep: default 5
// - Skip: [(blank), ??]
// - Power: power in watts with W suffix, e.g. [1.0W, 4.0W, 50W]
// - Comment: blank by default
// - URCALL: blank by default
// - RPT1CALL: blank by default
// - RPT2CALL: blank by default
// - DVCODE: blank by default

