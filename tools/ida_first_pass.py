"""Headless IDA first pass for the PANaHON arm64 Flutter library."""

import json
import os
from pathlib import Path

import idaapi
import ida_bytes
import ida_ida
import ida_segment
import idautils
import idc


def main() -> None:
    output = Path(os.environ.get("IDA_FIRST_PASS_OUT", "ida-first-pass.json"))
    segments = []
    for start in idautils.Segments():
        segment = ida_segment.getseg(start)
        if segment is None:
            continue
        segments.append(
            {
                "name": ida_segment.get_segm_name(segment),
                "start": hex(segment.start_ea),
                "end": hex(segment.end_ea),
                "permissions": {
                    "read": bool(segment.perm & ida_segment.SEGPERM_READ),
                    "write": bool(segment.perm & ida_segment.SEGPERM_WRITE),
                    "execute": bool(segment.perm & ida_segment.SEGPERM_EXEC),
                },
            }
        )

    keyword_hits = []
    keywords = ("forecast", "weather", "alert", "observation", "radar", "location", "panahon")
    for string_item in idautils.Strings():
        value = str(string_item)
        lowered = value.lower()
        if any(keyword in lowered for keyword in keywords):
            keyword_hits.append(
                {
                    "address": hex(int(string_item.ea)),
                    "text": value[:240],
                }
            )
            if len(keyword_hits) >= 200:
                break

    function_count = sum(1 for _ in idautils.Functions())
    result = {
        "tool": "IDA Pro headless first pass",
        "ida_version": idaapi.get_kernel_version(),
        "input": str(Path(idc.get_input_file_path()).resolve()),
        "architecture": ida_ida.inf_get_procname(),
        "function_count": function_count,
        "segments": segments,
        "keyword_string_hits": keyword_hits,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    print("IDA_FIRST_PASS_OK", output)
    idaapi.qexit(0)


main()
