# docs/radios/tyt_mduv390

`tyt_mduv390` allows for generation of TYT MD-UV390 CPS exports. Parsing is not supported due to extremely limited CPS export functionality.

## Limitations

The MD-UV380 CPS V2.41 (current as of 2025-02-25) has severe limitations for CSV import/export. Only the channels and talkgroups (contacts) support import/export. Zones, scanlists, radio IDs, and talkgroup lists must all be managed manually. It is highly recommended that you flash this radio with [OpenGD77](https://www.opengd77.com/) or [OpenRTX](https://openrtx.org/). Nevertheless, the factory CPS V2.41 is supported (model `tyt_mduv390`) if you must use the stock firmware.

# TYT MD-UV390

## Specs

| Specs | |
|:-|--|
| TX Frequencies | 136-174 MHz, 400-480 MHz |
| RX Frequencies | " |
| Emission Designators | 16K0F3E/11K0F3E<br>7K60FXD/7K60FXE  |
| Power | 5W, 2.5W, 1W |

## Properties

| Property | Value | Source |
|:-|:-|--|
| Modes | FM, NFM, DMR | specs |
| Channels | 3000 | specs |
| Ch. name len | 16 |
| Zones | 250 | specs |
| Zone name len | 16 | CPS V2.41 |
| Ch./zone | 64 | specs |
| Scanlists | ?? |  |
| Scanlist name len | 16 | CPS V2.41 |
| Ch./scanlist | ?? |
| DMR Talkgroups | 10,000 | specs |
| DMR Talkgroup name len | 16 | CPS V2.41 |
| DMR Talkgroup lists | ?? |  |
| DMR Talkgroup list name len | 16 | CPS V2.41 |
| DMR Talkgroups/talkgroup list | ?? |
| DMR IDs | 4? | CPS V2.41 |
| DMR ID name len | n/a | CPS V2.41 |
