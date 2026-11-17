# Mobile core

This crate contains the small Rust boundary used by the native Android client.

- `normalize_snapshot` validates the Rust weather-service envelope.
- `background_profile` derives an original weather and day/night visual profile.
- The `jni` feature exports `NativeBridge.nativeNormalizeSnapshot` and `NativeBridge.nativeBackgroundProfile`.

The crate does not perform network requests. Android owns the HTTP client and cache, so a failed refresh can display the last trusted snapshot with its stale metadata.

