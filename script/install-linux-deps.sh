#!/usr/bin/env bash
# GPUI's Linux build and runtime packages, verbatim from gpui-component's
# script/install-linux.sh (tested on Ubuntu 24.04).
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  gcc g++ clang libfontconfig-dev libwayland-dev \
  libwebkit2gtk-4.1-dev libxkbcommon-x11-dev libx11-xcb-dev \
  libssl-dev libzstd-dev libasound2-dev \
  vulkan-validationlayers libvulkan1
