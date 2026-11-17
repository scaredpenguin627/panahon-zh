# Rust weather service

`panahon-weather-api` is the backend service layer for the selected public data sources. It does not contain UI code.

## Run

```text
cargo run -p panahon-weather-api
```

The default listener is `0.0.0.0:8788`. Set `PANAHON_BIND` to change it. Upstream roots and the EMB bridge endpoint can be overridden with `PAGASA_API_ROOT`, `PAGASA_MAP_ROOT`, `OPEN_METEO_ROOT`, `OPEN_METEO_AIR_ROOT`, and `EMB_AQI_URL`.

## API

JSON endpoints return an envelope with `data`, `meta`, and `errors`. `meta.source` identifies the providers, `meta.quality` describes provenance, and `meta.stale` is set when a cached result was used after an upstream failure.

- `GET /health`
- `GET /api/v1/version`
- `GET /api/v1/locations/search?q=man`
- `GET /api/v1/weather?lat=14.5995&lon=120.9842`
- `GET /api/v1/air-quality?lat=14.5995&lon=120.9842`
- `GET /api/v1/alerts?lat=14.5995&lon=120.9842`
- `GET /api/v1/maps/radar`
- `GET /api/v1/maps/radar/frames/0`

The Rust service reads the DENR-EMB Power BI report directly by default using the recipes already checked into the repository. `EMB_AQI_URL` can point at an internal bridge when a deployment wants to reuse the existing Node collector. CAMS through Open-Meteo remains the fallback for NCR EMB failures and the primary source outside the NCR bounds.

