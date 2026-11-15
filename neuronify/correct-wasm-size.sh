#!/bin/sh
set -eu
cd "$TRUNK_STAGING_DIR"
size=$(($(wc -c < "$(ls ./*_bg.wasm)")))
sed "s/\(_bg\.wasm', *\)[0-9]*/\1$size/" index.html > index.html.new
grep -q "_bg\.wasm', *$size," index.html.new || {
  echo "correct-wasm-size.sh: found no wasm size in index.html to correct" >&2
  exit 1
}
mv index.html.new index.html
