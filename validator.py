from radios.radio_helper import RadioHelper
from log_helper import LogHelper as lh

def validate(json_data):
    if not json_data:
        lh.cprint(lh.ANSI_C_RED, "Error: JSON data is empty")
        return False

    if len(json_data["channels"]) == 0:
        lh.cprint(lh.ANSI_C_RED, "Error: No channels found in the CSV file")
        return
    else:
        lh.cprint(lh.ANSI_C_GRN, f"(validate) Found {len(json_data['channels'])} channels")

    # Validate channels
    for channel in json_data["channels"]:
        if channel["freq_rx"] != channel["freq_tx"]:
            if RadioHelper.is_2m(channel["freq_rx"]) and RadioHelper.is_2m(channel["freq_tx"]):
                # 2m monoband, offset should be +/- 600 kHz
                if abs(channel["freq_rx"] - channel["freq_tx"]) != 600_000:
                    lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Ch {channel['index']:5d} has unusual offset: {RadioHelper.hzInt_to_mhzStr(channel['freq_rx'] - channel['freq_tx'])} MHz")
                    lh.cprint(lh.ANSI_C_YLW, f"(validate)          name = {channel['name']}, rx = {RadioHelper.hzInt_to_mhzStr(channel['freq_rx'])} MHz, tx = {RadioHelper.hzInt_to_mhzStr(channel['freq_tx'])} MHz")
            if RadioHelper.is_70cm(channel["freq_rx"]) and RadioHelper.is_70cm(channel["freq_tx"]):
                # 70cm monoband, offset should be +/- 5 MHz
                if abs(channel["freq_rx"] - channel["freq_tx"]) != 5_000_000:
                    lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Ch {channel['index']:5d} has unusual offset: {RadioHelper.hzInt_to_mhzStr(channel['freq_rx'] - channel['freq_tx'])} MHz")
                    lh.cprint(lh.ANSI_C_YLW, f"(validate)          name = {channel['name']}, rx = {RadioHelper.hzInt_to_mhzStr(channel['freq_rx'])} MHz, tx = {RadioHelper.hzInt_to_mhzStr(channel['freq_tx'])} MHz")
            if (RadioHelper.is_2m(channel["freq_rx"]) and RadioHelper.is_70cm(channel["freq_tx"])) or (RadioHelper.is_70cm(channel["freq_rx"]) and RadioHelper.is_2m(channel["freq_tx"])):
                # Warn on crossband
                lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Ch {channel['index']:5d} is crossband: rx={RadioHelper.hzInt_to_mhzStr(channel['freq_rx'])}, tx={RadioHelper.hzInt_to_mhzStr(channel['freq_tx'])}")
                lh.cprint(lh.ANSI_C_YLW, f"(validate)          name = {channel['name']}")

    # Validate zones
    if "zones" in json_data:
        for zone in json_data["zones"]:
            if len(zone["channels"]) < 1:
                lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Zone {zone['name']} has no channels")
        for zone in json_data["zones"]:
            for channel in zone["channels"]:
                # Search for channel by name in the list of channels
                found = False
                for c in json_data["channels"]:
                    if c["name"] == channel:
                        found = True
                        break
                if not found:
                    lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Zone {zone['name']} contains orphan channel: {channel}")

    for channel in json_data["channels"]:
        if channel["mode"] == "DMR":
            if (not channel["dmr"]["contact"]) and (not channel["dmr"]["tg_list"]):
                lh.cprint(lh.ANSI_C_YLW, f"(validate) Warning: Ch {channel['index']:5d} has no talkgroup or talkgroup list")
                lh.cprint(lh.ANSI_C_YLW, f"(validate)          name = {channel['name']}")
    return True
