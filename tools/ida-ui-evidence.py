"""Read-only UI symbol inventory through the official Hex-Rays IDA MCP server.

Usage: python tools/ida-ui-evidence.py <database> <output.json>
Requires ida-mcp on PYTHONPATH and an installed IDA Pro license.
Only class/resource names and counts are exported, never function bodies.
"""
import json
import os
import pathlib
import queue
import subprocess
import sys
import threading


def main():
    target, output = sys.argv[1:3]
    output = pathlib.Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.with_suffix('.stderr.log').open('w', encoding='utf-8') as log:
        process = subprocess.Popen(
            [sys.executable, '-c', 'from ida_mcp.cli import main; main()', 'stdio', '--agent=panahon-evidence'],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
            text=True, encoding='utf-8', env=os.environ.copy(),
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
        )
        lines = queue.Queue()
        def receive():
            for line in process.stdout:
                lines.put(line)
        threading.Thread(target=receive, daemon=True).start()
        counter = 0
        def rpc(method, params):
            nonlocal counter
            counter += 1
            process.stdin.write(json.dumps({'jsonrpc':'2.0', 'id':counter, 'method':method, 'params':params}) + '\n')
            process.stdin.flush()
            while True:
                raw = lines.get(timeout=240)
                try:
                    response = json.loads(raw)
                except ValueError:
                    continue
                if response.get('id') == counter:
                    if 'error' in response:
                        raise RuntimeError(response['error'])
                    return response.get('result')
        record = {}
        try:
            record['initialize'] = rpc('initialize', {'protocolVersion':'2024-11-05', 'capabilities':{}, 'clientInfo':{'name':'panahon-evidence', 'version':'1'}})
            process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
            process.stdin.flush()
            record['tools'] = rpc('tools/list', {})
            print('MCP initialized; tools listed', flush=True)
            record['open'] = rpc('tools/call', {'name':'open_database', 'arguments':{'path':target}})
            print('Database open response received', flush=True)
            record['reference'] = rpc('tools/call', {'name':'reference', 'arguments':{'query':'strings'}})
            code = '''import ida_nalt, ida_ida, ida_funcs, idautils
keywords = ('WeatherHome', 'CurrentTemp', 'CurrentWeather', 'Hourly', 'ForecastWeather', 'LifeIndex', 'SunAndMoon', 'AirQuality', 'WeatherSun', 'PullDown', 'CityWeather', 'weather_hourly', 'weather_life', 'featured_map')
hits = [{'ea':hex(ea), 'name':name} for ea,name in idautils.Names() if any(k.lower() in name.lower() for k in keywords)]
strings = [{'ea':hex(s.ea), 'text':str(s)} for s in idautils.Strings() if any(k.lower() in str(s).lower() for k in keywords)]
{'input':ida_nalt.get_input_file_path(), 'processor':ida_ida.inf_get_procname(), 'function_count':ida_funcs.get_func_qty(), 'ui_names':hits[:150], 'ui_strings':strings[:150], 'body_exported':False}
'''
            record['inventory'] = rpc('tools/call', {'name':'execute_python', 'arguments':{'code':code, 'timeout':60}})
            print('UI name inventory received', flush=True)
            record['close'] = rpc('tools/call', {'name':'close_database', 'arguments':{}})
        finally:
            output.write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding='utf-8')
            process.terminate()
            process.wait(timeout=15)
        print(f'Saved {output}', flush=True)


if __name__ == '__main__':
    main()
