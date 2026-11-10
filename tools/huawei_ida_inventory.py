"""Read-only IDA inventory for the Huawei Weather APK.

Run with IDA's native Python, for example:
  ida.exe -A -S".../huawei_ida_inventory.py" D:/huawei-weather-14-0-15-301.apk
"""

import json
import os

import idaapi
import ida_nalt
import idautils


def main():
    idaapi.auto_wait()
    info = idaapi.get_inf_structure()
    segments = []
    for index in range(idaapi.get_segm_qty()):
        segment = idaapi.getnseg(index)
        if segment:
            segments.append(
                {
                    "name": idaapi.get_segm_name(segment),
                    "start": hex(segment.start_ea),
                    "end": hex(segment.end_ea),
                    "perm": int(segment.perm),
                }
            )

    strings = []
    for item in idautils.Strings():
        value = str(item)
        if value and len(value) >= 4:
            strings.append(value[:240])
        if len(strings) >= 2000:
            break

    names = []
    for address, name in idautils.Names():
        if name and len(names) < 2000:
            names.append({"ea": hex(address), "name": name})

    result = {
        "input": ida_nalt.get_input_file_path(),
        "file_type": idaapi.get_file_type_name(),
        "processor": info.procname,
        "is_64bit": bool(info.is_64bit()),
        "segments": segments,
        "strings": strings,
        "names": names,
    }
    output = os.path.join(os.path.dirname(ida_nalt.get_input_file_path()), "huawei-ida-inventory.json")
    with open(output, "w", encoding="utf-8") as handle:
        json.dump(result, handle, ensure_ascii=False, indent=2)
    print("WROTE", output)
    idaapi.qexit(0)


if __name__ == "__main__":
    main()
