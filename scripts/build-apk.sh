#!/usr/bin/env bash
# Builds a signed release APK on Windows without needing Developer Mode.
#
# `tauri android build` compiles the Rust library, then tries to symlink it into the
# Gradle project, which Windows refuses unless Developer Mode is on. This script lets
# Tauri compile, copies the libraries instead, and runs Gradle with the Rust step skipped.
#
# Needs: JAVA_HOME (JDK 17+), ANDROID_HOME, NDK_HOME, and a C toolchain on PATH.
# Usage: scripts/build-apk.sh [aarch64|x86_64 ...]   (default: aarch64 x86_64)
set -euo pipefail

cd "$(dirname "$0")/.."
targets=("${@:-aarch64 x86_64}")
read -r -a targets <<< "${targets[*]}"

declare -A abi=( [aarch64]=arm64-v8a [x86_64]=x86_64 [armv7]=armeabi-v7a [i686]=x86 )
declare -A triple=( [aarch64]=aarch64-linux-android [x86_64]=x86_64-linux-android [armv7]=armv7-linux-androideabi [i686]=i686-linux-android )

jni=src-tauri/gen/android/app/src/main/jniLibs
for t in "${targets[@]}"; do
  echo "==> Rust ($t)"
  # Expected to fail at the symlink step; the library is already built by then.
  npx tauri android build --apk --target "$t" </dev/null >/dev/null 2>&1 || true
  lib="src-tauri/target/${triple[$t]}/release/libmoneywort_lib.so"
  [ -f "$lib" ] || { echo "Rust build for $t failed. Run: npx tauri android build --apk --target $t"; exit 1; }
  mkdir -p "$jni/${abi[$t]}"
  cp -f "$lib" "$jni/${abi[$t]}/"
done
cp -f src-tauri/tauri.conf.json src-tauri/gen/android/app/src/main/assets/tauri.conf.json

echo "==> Gradle"
cd src-tauri/gen/android
./gradlew.bat assembleUniversalRelease \
  -x rustBuildUniversalRelease -x rustBuildArm64Release -x rustBuildX86_64Release \
  -x rustBuildArmRelease -x rustBuildX86Release -q

apk=app/build/outputs/apk/universal/release/app-universal-release.apk
mkdir -p ../../../dist
cp -f "$apk" ../../../dist/moneywort.apk
echo "==> dist/moneywort.apk ($(du -h ../../../dist/moneywort.apk | cut -f1))"
