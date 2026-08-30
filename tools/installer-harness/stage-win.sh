#!/bin/sh
# The Windows release, staged locally: a .zip holding one ank.exe, and the
# sha256 beside it, in the layout install.ps1 unpacks.
set -eu

root=$PWD/.stage
tag=v9.9.9-preview
target=x86_64-pc-windows-msvc
dir=ank-9.9.9-preview-${target}
archive=${dir}.zip

mkdir -p "$root/$tag" "$root/winbuild/$dir"

# A stand-in that answers --version, which is the one thing install.ps1 asks of
# what it installed. Executable with a shebang, because this preview runs on
# Linux and `& $destination --version` has to reach something that runs.
cat > "$root/winbuild/$dir/ank.exe" <<'FAKE'
#!/bin/sh
[ "${1:-}" = "--version" ] && echo "ank 9.9.9-preview" && exit 0
echo "ank: this is the staged stand-in the preview installs"
FAKE
chmod +x "$root/winbuild/$dir/ank.exe"

rm -f "$root/$tag/$archive"
# Python and not `zip`, which this machine does not have. The executable bit is
# written into the external attributes by hand: ZipFile does not carry a mode
# on its own, and a stand-in that unpacks unexecutable answers no --version.
python3 - "$root/winbuild" "$dir" "$root/$tag/$archive" <<'ZIP'
import os, sys, zipfile
build, dirname, out = sys.argv[1], sys.argv[2], sys.argv[3]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(os.path.join(build, dirname)):
        for name in files:
            full = os.path.join(root, name)
            arc = os.path.relpath(full, build)
            info = zipfile.ZipInfo(arc)
            info.external_attr = (0o755 << 16)
            with open(full, "rb") as f:
                z.writestr(info, f.read())
ZIP
sha256sum "$root/$tag/$archive" | awk '{print $1}' > "$root/$tag/$archive.sha256"

printf '%s\n' "$root/$tag/$archive" >&2
