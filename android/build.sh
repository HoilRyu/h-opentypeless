#!/bin/sh
set -eu
cd "$(dirname "$0")"
: "${JAVA_HOME:?Set JAVA_HOME to a JDK 17 or 21 installation}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export GRADLE_USER_HOME="${GRADLE_USER_HOME:-$HOME/.local/share/h-opentypeless/android-tools/gradle-home}"
exec ./gradlew --project-cache-dir "$HOME/.local/share/h-opentypeless/android-build/project-cache" "$@"
