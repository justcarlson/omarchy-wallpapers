#!/usr/bin/env bash
# Omarchy Wallpapers: curated backgrounds for every Omarchy theme.
#
#   curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash
#
# https://github.com/justcarlson/omarchy-wallpapers
set -euo pipefail

raw="${OMARCHY_WALLPAPERS_URL:-https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/${OMARCHY_WALLPAPERS_REF:-main}}"
backgrounds="${OMARCHY_BACKGROUNDS:-$HOME/.config/omarchy/backgrounds}"
cache="${XDG_CACHE_HOME:-$HOME/.cache}/omarchy-wallpapers"
state="${XDG_STATE_HOME:-$HOME/.local/state}/omarchy-wallpapers/installed"

usage() {
  cat <<'EOF'
Usage: install.sh [theme...] [--all | --check | --list | --remove]

  (no arguments)  Install the sets for the Omarchy themes you have
  theme...        Install only these sets, such as: gruvbox nord
  --all           Install every set, even for themes you do not have
  --check         Report missing or changed wallpapers; change nothing
  --list          List the sets and how many wallpapers each has
  --remove        Remove every wallpaper this script installed, and its cache

With curl, pass options after "bash -s --":
  curl -fsSL .../install.sh | bash -s -- gruvbox nord
EOF
}

if [[ -t 1 ]]; then
  bold=$'\e[1m' dim=$'\e[2m' green=$'\e[32m' red=$'\e[31m' reset=$'\e[0m'
else
  bold="" dim="" green="" red="" reset=""
fi
say() { printf '%s\n' "$*"; }
fail() { printf '%serror:%s %s\n' "$red" "$reset" "$*" >&2; exit 1; }

mode=install
all=false
requested=()
for arg in "$@"; do
  case $arg in
    -h | --help) usage; exit 0 ;;
    --all) all=true ;;
    --check) mode=check ;;
    --list) mode=list ;;
    --remove) mode=remove ;;
    -*) usage >&2; exit 2 ;;
    *) requested+=("$arg") ;;
  esac
done

for tool in curl sha256sum; do
  command -v "$tool" >/dev/null || fail "$tool is required"
done

temporary=""
cleanup() { [[ -z $temporary ]] || rm -f -- "$temporary"; }
trap cleanup EXIT

if [[ $mode == remove ]]; then
  removed=0
  if [[ -f $state ]]; then
    while IFS= read -r entry; do
      [[ $entry == */* ]] || continue
      if [[ -f $backgrounds/$entry ]]; then
        rm -f -- "${backgrounds:?}/$entry"
        removed=$((removed + 1))
      fi
      rmdir --ignore-fail-on-non-empty -- "$(dirname "$backgrounds/$entry")" 2>/dev/null || true
    done <"$state"
    rm -f -- "$state"
  fi
  rm -rf -- "${cache:?}"
  say "Removed $removed wallpapers and the download cache."
  exit 0
fi

# Run from a checkout: use its files. Run through curl: fetch the manifest.
source_dir=""
script=${BASH_SOURCE[0]:-}
if [[ -n $script && -f $script && -f $(dirname "$script")/wallpapers.tsv ]]; then
  source_dir=$(cd "$(dirname "$script")" && pwd)
  manifest=$(<"$source_dir/wallpapers.tsv")
else
  manifest=$(curl --fail --silent --show-error --location --retry 3 "$raw/wallpapers.tsv") ||
    fail "could not download the wallpaper list from $raw"
fi

declare -A count=()
order=()
while IFS=$'\t' read -r theme _ _ _; do
  [[ -z $theme || $theme == \#* ]] && continue
  [[ -n ${count[$theme]:-} ]] || order+=("$theme")
  count[$theme]=$((${count[$theme]:-0} + 1))
done <<<"$manifest"

# Themes on this machine: the user's own and Omarchy's built-in ones.
declare -A have=()
if [[ -n ${OMARCHY_PATH:-} ]]; then
  system=("$OMARCHY_PATH")
else
  system=(/usr/share/omarchy "$HOME/.local/share/omarchy")
fi
for dir in "$HOME/.config/omarchy/themes" "${system[@]/%//themes}"; do
  [[ -d $dir ]] || continue
  for theme in "$dir"/*/; do
    [[ -d $theme ]] && have[$(basename "$theme")]=1
  done
done

if [[ $mode == list ]]; then
  for theme in "${order[@]}"; do
    printf '%-18s %3s wallpapers  %s\n' "$theme" "${count[$theme]}" "${have[$theme]:+${dim}installed theme${reset}}"
  done
  exit 0
fi

declare -A selected=()
if ((${#requested[@]})); then
  for theme in "${requested[@]}"; do
    [[ -n ${count[$theme]:-} ]] || fail "no set named '$theme'. See: install.sh --list"
    selected[$theme]=1
  done
else
  for theme in "${order[@]}"; do
    if $all || [[ -n ${have[$theme]:-} ]]; then
      selected[$theme]=1
    fi
  done
fi
((${#selected[@]})) || fail "found no Omarchy themes. Name a set, or use --all"

matches() {
  [[ -f $1 && $(sha256sum "$1" | cut -d ' ' -f 1) == "$2" ]]
}

fetch() { # source checksum
  local source=$1 checksum=$2 cached="$cache/$2"
  matches "$cached" "$checksum" && return 0
  mkdir -p "$cache"
  temporary=$(mktemp "$cache/.download.XXXXXX")
  if [[ $source != https://* && -n $source_dir ]]; then
    cp -- "$source_dir/$source" "$temporary"
  else
    [[ $source == https://* ]] || source="$raw/$source"
    curl --fail --silent --show-error --location --retry 3 --output "$temporary" "$source" || { cleanup; temporary=""; return 1; }
  fi
  if ! matches "$temporary" "$checksum"; then
    say "${red}checksum mismatch${reset} for $source" >&2
    cleanup
    temporary=""
    return 1
  fi
  chmod 0644 "$temporary"
  mv -f -- "$temporary" "$cached"
  temporary=""
}

declare -A wanted=()
problems=0 added=0 total=0
current=""
while IFS=$'\t' read -r theme checksum file source; do
  [[ -z $theme || $theme == \#* || -z ${selected[$theme]:-} ]] && continue
  wanted[$theme/$file]=1
  total=$((total + 1))
  if [[ $theme != "$current" ]]; then
    current=$theme
    done_in_theme=0
  fi
  done_in_theme=$((done_in_theme + 1))
  [[ -t 1 && $mode == install ]] && printf '\r\e[2K%s%-18s%s %d/%d' "$bold" "$theme" "$reset" "$done_in_theme" "${count[$theme]}"
  target="$backgrounds/$theme/$file"
  matches "$target" "$checksum" && continue
  if [[ $mode == check ]]; then
    say "Missing or changed: $target"
    problems=$((problems + 1))
    continue
  fi
  if ! fetch "$source" "$checksum"; then
    [[ -t 1 ]] && printf '\r\e[2K'
    say "${red}skipped${reset} $theme/$file: download failed" >&2
    problems=$((problems + 1))
    continue
  fi
  mkdir -p "$backgrounds/$theme"
  # Hard-link from the cache so an image shared by two themes is stored once.
  temporary=$(mktemp "$backgrounds/$theme/.install.XXXXXX")
  ln -f -- "$cache/$checksum" "$temporary" 2>/dev/null || cp -- "$cache/$checksum" "$temporary"
  mv -f -- "$temporary" "$target"
  temporary=""
  added=$((added + 1))
done <<<"$manifest"
[[ -t 1 && $mode == install ]] && printf '\r\e[2K'

# Remove files this script installed earlier that the list has since dropped.
kept=()
if [[ -f $state ]]; then
  while IFS= read -r entry; do
    [[ $entry == */* ]] || continue
    theme=${entry%%/*}
    if [[ -n ${wanted[$entry]:-} ]]; then
      continue
    elif [[ -n ${selected[$theme]:-} || -z ${count[$theme]:-} ]]; then
      if [[ -f $backgrounds/$entry ]]; then
        if [[ $mode == check ]]; then
          say "Dropped from the collection: $backgrounds/$entry"
          problems=$((problems + 1))
          kept+=("$entry")
        else
          rm -f -- "${backgrounds:?}/$entry"
        fi
      fi
    else
      kept+=("$entry")
    fi
  done <"$state"
fi

if [[ $mode == install ]]; then
  mkdir -p "$(dirname "$state")"
  { printf '%s\n' "${kept[@]}"; printf '%s\n' "${!wanted[@]}"; } | grep -v '^$' | sort -u >"$state.tmp"
  mv -f -- "$state.tmp" "$state"
  if command -v omarchy-menu-images >/dev/null; then
    for theme in "${!selected[@]}"; do
      omarchy-menu-images --cache-only "$backgrounds/$theme" >/dev/null 2>&1 || true
    done
  fi
fi

if ((problems)); then
  [[ $mode == check ]] && fail "$problems problems. Run install.sh to fix them."
  fail "$problems wallpapers failed. Run the same command again to retry them."
fi

if [[ $mode == check ]]; then
  say "${green}All $total wallpapers across ${#selected[@]} themes are in place.${reset}"
else
  say "${green}${bold}$total wallpapers ready across ${#selected[@]} themes${reset} ${dim}($added new)${reset}"
  say "Press ${bold}Super + Ctrl + Space${reset} to pick a background for the current theme."
fi
