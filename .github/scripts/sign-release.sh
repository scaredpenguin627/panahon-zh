#!/usr/bin/env bash
set -euo pipefail
umask 077

tag="${GITHUB_REF_NAME:?}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]
version="${tag#v}"
tools="$ANDROID_HOME/build-tools/37.0.0"
source_dir="breezy-weather/app/build/outputs/apk/basic/release"
out="$RUNNER_TEMP/panahon-apks"
mkdir -p "$out"
key=$(mktemp "$RUNNER_TEMP/panahon-signing.XXXXXXXX")
trap 'rm -f "$key"' EXIT
printf '%s' "${SIGNING_KEY:?}" | base64 --decode > "$key"
unset SIGNING_KEY

# Public SHA-256 fingerprint of this project's release certificate.
fingerprint=ab1a327e665d8e17899b075240762e5541978dce09402b4ef3158135b2572dba
abis=(armeabi-v7a arm64-v8a x86 x86_64)
version_code=""
for abi in universal "${abis[@]}"; do
  source_apk="$source_dir/app-basic-$abi-release-unsigned.apk"
  apk="$out/panahon-zh-$abi-$tag.apk"
  "$tools/apksigner" sign --ks "$key" --ks-key-alias "${ALIAS:?}" \
    --ks-pass env:KEY_STORE_PASSWORD --key-pass env:KEY_PASSWORD \
    --v4-signing-enabled false --out "$apk" "$source_apk"
  "$tools/zipalign" -c -P 16 4 "$apk"
  certs=$("$tools/apksigner" verify --verbose --print-certs "$apk")
  verified_fingerprints=$(sed -n 's/^.*certificate SHA-256 digest: //p' <<< "$certs" | sort -u)
  [[ "$verified_fingerprints" == "$fingerprint" ]]
  grep -Fqx 'Number of signers: 1' <<< "$certs"
  badging=$("$tools/aapt2" dump badging "$apk")
  package=$(sed -n '/^package: /p' <<< "$badging")
  [[ "$package" == *"name='org.panahon.zh.weather'"* && "$package" == *"versionName='$version'"* ]]
  if grep -Fqx 'application-debuggable' <<< "$badging"; then
    echo "Release APK unexpectedly allows debugging" >&2
    exit 1
  fi
  code=$(sed -n "s/.*versionCode='\([0-9]*\)'.*/\1/p" <<< "$package")
  [[ -n "$code" && "$code" == "${version_code:-$code}" ]]
  version_code="$code"
  expected=("$abi")
  if [[ "$abi" == universal ]]; then expected=("${abis[@]}"); fi
  actual=$(unzip -Z1 "$apk" | awk -F/ '/^lib\/[^/]+\/[^/]+\.so$/ {print $2}' | sort -u)
  [[ "$actual" == "$(printf '%s\n' "${expected[@]}" | sort -u)" ]]
  echo "Verified $abi, version $version ($code), signer $fingerprint."
done

(cd "$out" && sha256sum *.apk > SHA256SUMS)
cat > "$RUNNER_TEMP/release-notes.md" <<EOF
简体中文 Android 正式版，基于 Breezy Weather。Pixel 10a 请选择 **arm64-v8a**；不确定设备架构时请选择 **universal**。其他架构可选择 armeabi-v7a、x86 或 x86_64。

正式版包名为 \`org.panahon.zh.weather\`。首次使用正式版时，可在调试版导出设置和地点，再在正式版导入。

发布证书 SHA-256：\`$fingerprint\`

各 APK 的 SHA-256 见随附的 SHA256SUMS。
EOF
