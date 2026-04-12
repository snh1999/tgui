#!/usr/bin/env bash
set -e

VERSION="2.3"
DEST="src-tauri/resources/tldr-pages-${VERSION}"

if [ -d "$DEST" ]; then
  echo "tldr-pages $VERSION already present, skipping"
  exit 0
fi

echo "Downloading tldr-pages $VERSION..."
curl -L "https://github.com/tldr-pages/tldr/releases/download/v${VERSION}/tldr-pages.zip" -o /tmp/tldr.zip
unzip -q /tmp/tldr.zip -d /tmp/tldr-extract
mkdir -p "$DEST"
mv "/tmp/tldr-extract"/* "$DEST/"
rm -rf /tmp/tldr.zip /tmp/tldr-extract

echo "Done → $DEST"
