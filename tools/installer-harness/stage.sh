#!/bin/sh
# A release that exists only here, so the whole of a successful install can be
# watched without waiting on GitHub. The same shape CI stages: an archive
# holding one executable named `ank`, and the sha256 beside it.
set -eu

here=$PWD
root=$here/.stage
tag=v9.9.9-preview
target=$(uname -m)
case $target in
  x86_64 | amd64) target=x86_64-unknown-linux-musl ;;
  aarch64 | arm64) target=aarch64-unknown-linux-musl ;;
esac
archive=ank-9.9.9-preview-${target}.tar.gz

rm -rf "$root"
mkdir -p "$root/$tag" "$root/build/ank-9.9.9-preview-${target}"

cat > "$root/build/ank-9.9.9-preview-${target}/ank" <<'FAKE'
#!/bin/sh
[ "${1:-}" = "--version" ] && echo "ank 9.9.9-preview" && exit 0
echo "ank: this is the staged stand-in the preview installs"
FAKE
chmod +x "$root/build/ank-9.9.9-preview-${target}/ank"

tar -C "$root/build" -czf "$root/$tag/$archive" "ank-9.9.9-preview-${target}"
sha256sum "$root/$tag/$archive" | awk '{print $1}' > "$root/$tag/$archive.sha256"

printf '%s\n' "$root/$tag/$archive" >&2
