#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
mkdir -p "$tmp_dir/bin" "$tmp_dir/fixtures"
: > "$tmp_dir/commands.log"
printf 'mock release binary\n' > "$tmp_dir/fixtures/ani-es-rs-linux-x86_64"
printf 'mock source build binary\n' > "$tmp_dir/fixtures/ani-es-rs-source"
if command -v sha256sum >/dev/null 2>&1; then
    digest="$(sha256sum "$tmp_dir/fixtures/ani-es-rs-linux-x86_64" | awk '{print $1}')"
else
    digest="$(shasum -a 256 "$tmp_dir/fixtures/ani-es-rs-linux-x86_64" | awk '{print $1}')"
fi
printf '%s  ani-es-rs-linux-x86_64\n' "$digest" > "$tmp_dir/fixtures/SHA256SUMS"
printf '%064d  ani-es-rs-linux-x86_64\n' 0 > "$tmp_dir/fixtures/SHA256SUMS.bad"

cat > "$tmp_dir/bin/uname" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == -m ]]; then echo "${ARCH_OVERRIDE:-x86_64}"; else echo Linux; fi
EOF
cat > "$tmp_dir/bin/curl" <<'EOF'
#!/usr/bin/env bash
printf 'curl %s\n' "$*" >> "$COMMAND_LOG"
output=''
url=''
write_out=0
while (($#)); do
    case "$1" in
        --output|-o) output="$2"; shift 2 ;;
        --write-out|-w) write_out=1; shift 2 ;;
        http*) url="$1"; shift ;;
        *) shift ;;
    esac
done
if ((write_out)); then
    printf '%s\n' 'https://github.com/olalekanob6/ani-es-rs/releases/tag/v9.8.7'
elif [[ "$url" == */SHA256SUMS ]]; then
    if [[ "${CHECKSUM_MODE:-good}" == bad ]]; then
        cp "$FIXTURES/SHA256SUMS.bad" "$output"
    else
        cp "$FIXTURES/SHA256SUMS" "$output"
    fi
elif [[ "$url" == */ani-es-rs-linux-x86_64 ]]; then
    cp "$FIXTURES/ani-es-rs-linux-x86_64" "$output"
else
    echo "Unexpected mocked download URL: $url" >&2
    exit 1
fi
EOF
cat > "$tmp_dir/bin/sudo" <<'EOF'
#!/usr/bin/env bash
printf 'sudo %s\n' "$*" >> "$COMMAND_LOG"
EOF
cat > "$tmp_dir/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" >> "$COMMAND_LOG"
mkdir -p target/release
cp "$FIXTURES/ani-es-rs-source" target/release/ani-es-rs
EOF
chmod +x "$tmp_dir/bin/"*

export COMMAND_LOG="$tmp_dir/commands.log"
export FIXTURES="$tmp_dir/fixtures"
export PATH="$tmp_dir/bin:$PATH"

if ! bash "$repo_root/install.sh" --install-dir "$tmp_dir/install" > "$tmp_dir/output.log" 2>&1; then
    cat "$tmp_dir/output.log" >&2
    exit 1
fi
if ! grep -q 'releases/latest' "$COMMAND_LOG"; then
    echo "FAIL: installer did not resolve the latest release" >&2
    cat "$COMMAND_LOG" >&2
    exit 1
fi
if [[ "$(cat "$tmp_dir/install/ani-es-rs")" != 'mock release binary' ]]; then
    echo "FAIL: installer did not install the downloaded release asset" >&2
    exit 1
fi
if grep -q '^sudo ' "$COMMAND_LOG"; then
    echo "FAIL: default install invoked sudo" >&2
    cat "$COMMAND_LOG" >&2
    exit 1
fi

rm -f "$tmp_dir/install/ani-es-rs"
if CHECKSUM_MODE=bad bash "$repo_root/install.sh" --install-dir "$tmp_dir/install" > "$tmp_dir/bad-output.log" 2>&1; then
    echo "FAIL: installer accepted a mismatched checksum" >&2
    exit 1
fi
if [[ -e "$tmp_dir/install/ani-es-rs" ]]; then
    echo "FAIL: checksum failure left an installed binary" >&2
    exit 1
fi

: > "$COMMAND_LOG"
if ! ARCH_OVERRIDE=aarch64 bash "$repo_root/install.sh" --from-source --install-dir "$tmp_dir/source-install" > "$tmp_dir/source-output.log" 2>&1; then
    cat "$tmp_dir/source-output.log" >&2
    exit 1
fi
if grep -q '^curl ' "$COMMAND_LOG"; then
    echo "FAIL: source install unexpectedly contacted the release server" >&2
    cat "$COMMAND_LOG" >&2
    exit 1
fi
if [[ "$(cat "$tmp_dir/source-install/ani-es-rs")" != 'mock source build binary' ]]; then
    echo "FAIL: source install did not install the compiled binary" >&2
    exit 1
fi

if bash "$repo_root/install.sh" --interactive --install-dir "$tmp_dir/install" > "$tmp_dir/non-tty.log" 2>&1; then
    echo "FAIL: interactive mode accepted a non-interactive terminal" >&2
    exit 1
fi
mkdir -p "$tmp_dir/uninstall"
printf 'binary' > "$tmp_dir/uninstall/ani-es-rs"
printf 'keep' > "$tmp_dir/uninstall/user-data.db"
: > "$COMMAND_LOG"
bash "$repo_root/install.sh" --uninstall --install-dir "$tmp_dir/uninstall" > "$tmp_dir/uninstall.log" 2>&1
[[ ! -e "$tmp_dir/uninstall/ani-es-rs" ]] || { echo "FAIL: uninstall left the binary" >&2; exit 1; }
[[ "$(cat "$tmp_dir/uninstall/user-data.db")" == keep ]] || { echo "FAIL: uninstall removed unrelated data" >&2; exit 1; }
if grep -q '^curl ' "$COMMAND_LOG"; then
    echo "FAIL: uninstall contacted the release server" >&2
    exit 1
fi

echo "PASS: verified release install, source fallback, non-TTY behavior, and safe uninstall"
