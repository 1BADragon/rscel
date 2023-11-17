#!/usr/bin/env sh

# first delete the package folders...
rm -rf pkg/ pkg_cjs/

# build the web target
RUSTFLAGS="-C opt-level=s" wasm-pack build --target web --release --features wasm

# next build the cjs target
RUSTFLAGS="-C opt-level=s" wasm-pack build --out-dir pkg_cjs --out-name rscel_cjs \
  --target nodejs --release --features wasm

cp -v pkg_cjs/{rscel_cjs_bg.wasm,rscel_cjs.js,rscel_cjs.d.ts} pkg/

pushd pkg
PACKAGE_JSON=$(jq '.name="@cedarai/rscel"' package.json)

PACKAGE_JSON=$(jq '.files[.files | length] |= . + "rscel_cjs_bg.wasm"' <<< "$PACKAGE_JSON")
PACKAGE_JSON=$(jq '.files[.files | length] |= . + "rscel_cjs.js"' <<< "$PACKAGE_JSON")
PACKAGE_JSON=$(jq '.files[.files | length] |= . + "rscel_cjs.d.ts"' <<< "$PACKAGE_JSON")
PACKAGE_JSON=$(jq '.main = "rscel_cjs.js"' <<< "$PACKAGE_JSON") 

echo $PACKAGE_JSON | jq | tee package.json
popd 

#rm -r pkg_cjs/
