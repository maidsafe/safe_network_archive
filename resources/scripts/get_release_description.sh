#!/usr/bin/env bash

testnet_version=$( \
  grep "^version" < sn_testnet/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_updater_version=$( \
  grep "^version" < sn_updater/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_fault_detection_version=$( \
  grep "^version" < sn_fault_detection/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_interface_version=$( \
  grep "^version" < sn_interface/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_comms_version=$( \
  grep "^version" < sn_comms/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_client_version=$( \
  grep "^version" < sn_client/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
safenode_version=$(grep "^version" < sn_node/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_api_version=$(grep "^version" < sn_api/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
safe_version=$(grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
This release of Safe Network consists of:
* Safe Updater v__SN_UPDATER_VERSION__
* Safe Node Fault Detection v__SN_FAULT_DETECTION_VERSION__
* Safe Network Interface v__SN_INTERFACE_VERSION__
* Safe Node Comms v__SN_COMMS_VERSION__
* Safe Client v__SN_CLIENT_VERSION__
* Safe Node v__SAFENODE_VERSION__
* Safe API v__SN_API_VERSION__
* Safe CLI v__SAFE_VERSION__
* Testnet v__TESTNET_VERSION__

## Safe Updater Changelog

__SN_UPDATER_CHANGELOG_TEXT__

## Safe Network Interface Changelog

__SN_INTERFACE_CHANGELOG_TEXT__

## Safe Node Fault Detection Changelog

__SN_FAULT_DETECTION_CHANGELOG_TEXT__

## Safe Node Comms Changelog

__SN_COMMS_CHANGELOG_TEXT__

## Safe Node Changelog

__SAFENODE_CHANGELOG_TEXT__

## Safe Client Changelog

__SN_CLIENT_CHANGELOG_TEXT__

## Safe API Changelog

__SN_API_CHANGELOG_TEXT__

## Safe CLI Changelog

__SAFE_CHANGELOG_TEXT__

## Testnet Changelog

__TESTNET_CHANGELOG_TEXT__

## SHA-256 checksums for safenode archives:
```
Linux
zip: SAFENODE_ZIP_LINUX_CHECKSUM
tar.gz: SAFENODE_TAR_LINUX_CHECKSUM

macOS
zip: SAFENODE_ZIP_MACOS_CHECKSUM
tar.gz: SAFENODE_TAR_MACOS_CHECKSUM

Windows
zip: SAFENODE_ZIP_WIN_CHECKSUM
tar.gz: SAFENODE_TAR_WIN_CHECKSUM

ARM
zip: SAFENODE_ZIP_ARM_CHECKSUM
tar.gz: SAFENODE_TAR_ARM_CHECKSUM

ARMv7
zip: SAFENODE_ZIP_ARMv7_CHECKSUM
tar.gz: SAFENODE_TAR_ARMv7_CHECKSUM

Aarch64
zip: SAFENODE_ZIP_AARCH64_CHECKSUM
tar.gz: SAFENODE_TAR_AARCH64_CHECKSUM
```

## SHA-256 checksums for safe archives:
```
Linux
zip: SAFE_ZIP_LINUX_CHECKSUM
tar.gz: SAFE_TAR_LINUX_CHECKSUM

macOS
zip: SAFE_ZIP_MACOS_CHECKSUM
tar.gz: SAFE_TAR_MACOS_CHECKSUM

Windows
zip: SAFE_ZIP_WIN_CHECKSUM
tar.gz: SAFE_TAR_WIN_CHECKSUM

ARM
zip: SAFE_ZIP_ARM_CHECKSUM
tar.gz: SAFE_TAR_ARM_CHECKSUM

ARMv7
zip: SAFE_ZIP_ARMv7_CHECKSUM
tar.gz: SAFE_TAR_ARMv7_CHECKSUM

Aarch64
zip: SAFE_ZIP_AARCH64_CHECKSUM
tar.gz: SAFE_TAR_AARCH64_CHECKSUM
```

## SHA-256 checksums for testnet archives:
```
Linux
zip: TESTNET_ZIP_LINUX_CHECKSUM
tar.gz: TESTNET_TAR_LINUX_CHECKSUM

macOS
zip: TESTNET_ZIP_MACOS_CHECKSUM
tar.gz: TESTNET_TAR_MACOS_CHECKSUM

Windows
zip: TESTNET_ZIP_WIN_CHECKSUM
tar.gz: TESTNET_TAR_WIN_CHECKSUM

ARM
zip: TESTNET_ZIP_ARM_CHECKSUM
tar.gz: TESTNET_TAR_ARM_CHECKSUM

ARMv7
zip: TESTNET_ZIP_ARMv7_CHECKSUM
tar.gz: TESTNET_TAR_ARMv7_CHECKSUM

Aarch64
zip: TESTNET_ZIP_AARCH64_CHECKSUM
tar.gz: TESTNET_TAR_AARCH64_CHECKSUM
```
EOF

safenode_zip_linux_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
safenode_zip_macos_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
safenode_zip_win_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
safenode_zip_arm_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
safenode_zip_armv7_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
safenode_zip_aarch64_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
safenode_tar_linux_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
safenode_tar_macos_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
safenode_tar_win_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
safenode_tar_arm_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
safenode_tar_armv7_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
safenode_tar_aarch64_checksum=$(sha256sum \
    "./deploy/safenode/safenode-$safenode_version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

safe_zip_linux_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
safe_zip_macos_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
safe_zip_win_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
safe_zip_arm_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
safe_zip_armv7_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
safe_zip_aarch64_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
safe_tar_linux_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
safe_tar_macos_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
safe_tar_win_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
safe_tar_arm_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
safe_tar_armv7_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
safe_tar_aarch64_checksum=$(sha256sum \
    "./deploy/safe/safe-$safe_version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

testnet_zip_linux_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
testnet_zip_macos_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
testnet_zip_win_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
testnet_zip_arm_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
testnet_zip_armv7_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
testnet_zip_aarch64_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
testnet_linux_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
testnet_macos_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
testnet_win_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
testnet_arm_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
testnet_armv7_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
testnet_aarch64_checksum=$(sha256sum \
    "./deploy/testnet/testnet-$testnet_version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

release_description=$(sed "s/__SN_UPDATER_VERSION__/$sn_updater_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_FAULT_DETECTION_VERSION__/$sn_fault_detection_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_INTERFACE_VERSION__/$sn_interface_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_COMMS_VERSION__/$sn_comms_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_CLIENT_VERSION__/$sn_client_version/g" <<< "$release_description")
release_description=$(sed "s/__SAFENODE_VERSION__/$safenode_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_API_VERSION__/$sn_api_version/g" <<< "$release_description")
release_description=$(sed "s/__SAFE_VERSION__/$safe_version/g" <<< "$release_description")
release_description=$(sed "s/__TESTNET_VERSION__/$testnet_version/g" <<< "$release_description")

release_description=$(sed "s/SAFENODE_ZIP_LINUX_CHECKSUM/$safenode_zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFENODE_ZIP_MACOS_CHECKSUM/$safenode_zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFENODE_ZIP_WIN_CHECKSUM/$safenode_zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_ZIP_ARM_CHECKSUM=$safenode_zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_ZIP_ARMv7_CHECKSUM=$safenode_zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_ZIP_AARCH64_CHECKSUM=$safenode_zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/SAFENODE_TAR_LINUX_CHECKSUM/$safenode_tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFENODE_TAR_MACOS_CHECKSUM/$safenode_tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFENODE_TAR_WIN_CHECKSUM/$safenode_tar_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_TAR_ARM_CHECKSUM=$safenode_tar_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_TAR_ARMv7_CHECKSUM=$safenode_tar_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFENODE_TAR_AARCH64_CHECKSUM=$safenode_tar_aarch64_checksum=g" <<< "$release_description")

release_description=$(sed "s/SAFE_ZIP_LINUX_CHECKSUM/$safe_zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFE_ZIP_MACOS_CHECKSUM/$safe_zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFE_ZIP_WIN_CHECKSUM/$safe_zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SAFE_ZIP_ARM_CHECKSUM=$safe_zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFE_ZIP_ARMv7_CHECKSUM=$safe_zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFE_ZIP_AARCH64_CHECKSUM=$safe_zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/SAFE_TAR_LINUX_CHECKSUM/$safe_tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFE_TAR_MACOS_CHECKSUM/$safe_tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SAFE_TAR_WIN_CHECKSUM/$safe_tar_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SAFE_TAR_ARM_CHECKSUM=$safe_tar_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFE_TAR_ARMv7_CHECKSUM=$safe_tar_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SAFE_TAR_AARCH64_CHECKSUM=$safe_tar_aarch64_checksum=g" <<< "$release_description")

release_description=$(sed "s/__TESTNET_VERSION__/$testnet_version/g" <<< "$release_description")
release_description=$(sed "s/TESTNET_ZIP_LINUX_CHECKSUM/$testnet_zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TESTNET_ZIP_MACOS_CHECKSUM/$testnet_zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TESTNET_ZIP_WIN_CHECKSUM/$testnet_zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=TESTNET_ZIP_ARM_CHECKSUM=$testnet_zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=TESTNET_ZIP_ARMv7_CHECKSUM=$testnet_zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=TESTNET_ZIP_AARCH64_CHECKSUM=$testnet_zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/TESTNET_TAR_LINUX_CHECKSUM/$testnet_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TESTNET_TAR_MACOS_CHECKSUM/$testnet_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TESTNET_TAR_WIN_CHECKSUM/$testnet_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=TESTNET_TAR_ARM_CHECKSUM=$testnet_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=TESTNET_TAR_ARMv7_CHECKSUM=$testnet_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=TESTNET_TAR_AARCH64_CHECKSUM=$testnet_aarch64_checksum=g" <<< "$release_description")

echo "$release_description"
