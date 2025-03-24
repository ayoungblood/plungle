# Supported Radios

| Make | Model | Freqs/Modes | Status |
|:-----|:------|:-----------:|:-------|
| Ailunce | HD1 | VHF/UHF FM/DMR | Partially supported |
| Alinco | DJ-MD5T* | VHF/UHF FM/DMR | Supported |
| Anytone | AT-D878UV* | VHF/UHF FM/DMR | Supported, also via qdmr |
| TYT | MD-UV3*0 | VHF/UHF FM/DMR | Partially supported, also via qdmr |


Also, the following programs are supported:

* [Chirp](https://chirpmyradio.com/projects/chirp/wiki/Home)
* [OpenGD77](https://www.opengd77.com/) - currently only the Retevis RT3S is supported for OpenGD77
* [Codeplug Editor](https://www.cpeditor.co.uk/) by David MM7DBT

Codeplugs from any of these programs can be read/written, although this has not been exhaustively tested.

## Properties

| Radio | Channels | Zones | Scanlists | Talkgroups | Channel name | Channels/zone | Channels/scanlist | Talkgroup lists | Talkgroups/talkgroup lists | Zone name |
|:------|---------:|------:|----------:|-----------:|-------------:|--------------:|------------------:|----------------:| --------------------------:|----------:|
| Ailunce HD1 | 3000 |
| Alinco DJ-MD5TGP | 4000 | 250 | 250 | 10,000 | 16 | 250 | 250 |
| Anytone AT-D878UVII | 4000 |
| Retevis RT3S | 3000 |
| Retevis RT3S (OpenGD77) | 1024 | 68 | - | 1024 | 16 | 80 | - | 76 | 32 | 16 |

### Ailunce HD1

Based on V3.03 CPS

* TX Frequencies: 136 - 174 MHz, 400 - 480 MHz
* Supported modes: FM, NFM, DMR
* Max channels: 3000
* Max channel name length: 14 (confirmed in CPS)
* Max zones: 256 (confirmed in CPS)
* Max zone name length: 16 (confirmed in CPS)
* Max channels per zone: 64
* Max talkgroups: 1000 (probably)
* Max talkgroup name length: 16 (confirmed in CPS)
* Max talkgroup lists: 255 (confirmed in CPS)
* Max talkgroups per talkgroup list: 33

### Alinco DJ-MD5TGP

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

### Anytone AT-D878UVII

* TX Frequencies: 136 - 174 MHz, 400 - 480 MHz
* CTCSS: 62.5 - 254.1 + custom (50 - 260)
* Supported modes: FM, NFM, DMR
* Max channels: 4000
* Max channel name length: 16
* Max zones: 250
* Max zone name length: 16
* Max channels per zone: 250
* Max scanlists: ??
* Max scanlist name length: 16
* Max channels per scanlist: ??
* Max talkgroups: 10000
* Max talkgroup name length: 16
* Max talkgroup lists: ??
* Max talkgroups per talkgroup list: ??

### Retevis RT3S

### Retevis RT3S (OpenGD77)

* TX Frequencies: 144 - 148 MHz, 420 - 450 MHz
* Supported modes: FM, NFM, DMR
* Max channels: 1024
* Max channel name length: 16
* Max zones: 68
* Max zone name length: 16
* Max channels per zone: 80
* Max talkgroups: 1024
* Max talkgroup name length: ??
* Max talkgroup lists: 76
* Max talkgroups per talkgroup list: 32

### Radioddity DB25-D

* TX Frequencies: 136 - 174 MHz, 400 - 480 MHz
* Supported modes: FM, NFM, DMR
* Max channels: 4000
* Max channel name length: 10
* Max zones: 4000 - channel count
* Max zone name length: 10
* Max channels per zone: 3999
* Max talkgroups: 2000
* Max talkgroup name length: 10
* Max talkgroup lists: ??
* Max talkgroups per talkgroup list: ??
