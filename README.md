<a href="https://justcarlson.github.io/omarchy-wallpapers/"><img src="docs/hero.jpg" alt="One wallpaper from each Omarchy theme in the collection"></a>

# Omarchy Wallpapers

<!-- stats:start -->
**295 wallpapers for 24 Omarchy themes**, each with an Aether palette.
<!-- stats:end -->

Every Omarchy theme ships with a few backgrounds. This adds a curated set to
each one: photographs, illustration and public-domain paintings, chosen
because their colours match the theme. Gruvbox gets autumn rivers,
tangerines and Cabanel's *Ophelia*. Nord gets slate-blue illustrations of
moonlit hills and night cities, and a stormy John Martin sea.

```sh
curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash
```

Then press <kbd>Super</kbd> <kbd>Ctrl</kbd> <kbd>Space</kbd> and pick one.

**[Browse the gallery](https://justcarlson.github.io/omarchy-wallpapers/)**
to see every wallpaper with its palette, or scroll down.

## What the installer does

It downloads the sets for the themes you have into
`~/.config/omarchy/backgrounds/<theme>/`, the folder Omarchy's background
picker reads beside the theme's own images. Every file is checked against a
SHA-256 in [`wallpapers.tsv`](wallpapers.tsv) before it is installed. An image
that two themes share is stored once. Your own backgrounds are never touched:
the installer only changes files it put there, and it lists them in
`~/.local/state/omarchy-wallpapers/installed`.

Run the same command again to update. New picks are added, and picks that
were dropped from the collection are removed.

Pass options after `bash -s --`:

```sh
curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash -s -- gruvbox nord   # just these sets
curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash -s -- --all          # every set, even for themes you don't have
curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash -s -- --check        # verify, change nothing
curl -fsSL https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/install.sh | bash -s -- --remove       # remove everything it installed
```

Prefer to read it first? It is [one Bash script](install.sh). It needs only
`curl` and `sha256sum`.

## Turn any wallpaper into a theme

[Aether](https://github.com/omacom/aether) is Omarchy's theme generator. It
builds a 16-colour palette from an image and applies it everywhere. Each
wallpaper here comes with an Aether blueprint in [`blueprints/`](blueprints):
the palette Aether made from that image, plus a link to the image.

In the [gallery](https://justcarlson.github.io/omarchy-wallpapers/), click
**Open in Aether** on any wallpaper. Or from a terminal:

```sh
aether --import-blueprint https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main/blueprints/gruvbox/alexandre-cabanel-ophelia.json --auto-apply
```

## The collection

<!-- gallery:start -->
<table>
<tr>
<td width="50%" valign="top"><a href="themes/catppuccin.md"><img src="docs/mosaics/catppuccin.jpg" alt="12 Catppuccin wallpapers"></a><br><a href="themes/catppuccin.md"><b>Catppuccin</b></a> · 12 wallpapers</td>
<td width="50%" valign="top"><a href="themes/catppuccin-latte.md"><img src="docs/mosaics/catppuccin-latte.jpg" alt="14 Catppuccin Latte wallpapers"></a><br><a href="themes/catppuccin-latte.md"><b>Catppuccin Latte</b></a> · 14 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/dracula.md"><img src="docs/mosaics/dracula.jpg" alt="10 Dracula wallpapers"></a><br><a href="themes/dracula.md"><b>Dracula</b></a> · 10 wallpapers</td>
<td width="50%" valign="top"><a href="themes/ethereal.md"><img src="docs/mosaics/ethereal.jpg" alt="11 Ethereal wallpapers"></a><br><a href="themes/ethereal.md"><b>Ethereal</b></a> · 11 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/everforest.md"><img src="docs/mosaics/everforest.jpg" alt="15 Everforest wallpapers"></a><br><a href="themes/everforest.md"><b>Everforest</b></a> · 15 wallpapers</td>
<td width="50%" valign="top"><a href="themes/flexoki-light.md"><img src="docs/mosaics/flexoki-light.jpg" alt="13 Flexoki Light wallpapers"></a><br><a href="themes/flexoki-light.md"><b>Flexoki Light</b></a> · 13 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/gruvbox.md"><img src="docs/mosaics/gruvbox.jpg" alt="16 Gruvbox wallpapers"></a><br><a href="themes/gruvbox.md"><b>Gruvbox</b></a> · 16 wallpapers</td>
<td width="50%" valign="top"><a href="themes/hackerman.md"><img src="docs/mosaics/hackerman.jpg" alt="10 Hackerman wallpapers"></a><br><a href="themes/hackerman.md"><b>Hackerman</b></a> · 10 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/jade-horse.md"><img src="docs/mosaics/jade-horse.jpg" alt="13 Jade Horse wallpapers"></a><br><a href="themes/jade-horse.md"><b>Jade Horse</b></a> · 13 wallpapers</td>
<td width="50%" valign="top"><a href="themes/kanagawa.md"><img src="docs/mosaics/kanagawa.jpg" alt="15 Kanagawa wallpapers"></a><br><a href="themes/kanagawa.md"><b>Kanagawa</b></a> · 15 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/last-horizon.md"><img src="docs/mosaics/last-horizon.jpg" alt="11 Last Horizon wallpapers"></a><br><a href="themes/last-horizon.md"><b>Last Horizon</b></a> · 11 wallpapers</td>
<td width="50%" valign="top"><a href="themes/lumon.md"><img src="docs/mosaics/lumon.jpg" alt="10 Lumon wallpapers"></a><br><a href="themes/lumon.md"><b>Lumon</b></a> · 10 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/lupine.md"><img src="docs/mosaics/lupine.jpg" alt="10 Lupine wallpapers"></a><br><a href="themes/lupine.md"><b>Lupine</b></a> · 10 wallpapers</td>
<td width="50%" valign="top"><a href="themes/matte-black.md"><img src="docs/mosaics/matte-black.jpg" alt="12 Matte Black wallpapers"></a><br><a href="themes/matte-black.md"><b>Matte Black</b></a> · 12 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/miasma.md"><img src="docs/mosaics/miasma.jpg" alt="13 Miasma wallpapers"></a><br><a href="themes/miasma.md"><b>Miasma</b></a> · 13 wallpapers</td>
<td width="50%" valign="top"><a href="themes/nord.md"><img src="docs/mosaics/nord.jpg" alt="14 Nord wallpapers"></a><br><a href="themes/nord.md"><b>Nord</b></a> · 14 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/retro-82.md"><img src="docs/mosaics/retro-82.jpg" alt="10 Retro 82 wallpapers"></a><br><a href="themes/retro-82.md"><b>Retro 82</b></a> · 10 wallpapers</td>
<td width="50%" valign="top"><a href="themes/ristretto.md"><img src="docs/mosaics/ristretto.jpg" alt="15 Ristretto wallpapers"></a><br><a href="themes/ristretto.md"><b>Ristretto</b></a> · 15 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/rose-pine.md"><img src="docs/mosaics/rose-pine.jpg" alt="14 Rose Pine wallpapers"></a><br><a href="themes/rose-pine.md"><b>Rose Pine</b></a> · 14 wallpapers</td>
<td width="50%" valign="top"><a href="themes/solitude.md"><img src="docs/mosaics/solitude.jpg" alt="11 Solitude wallpapers"></a><br><a href="themes/solitude.md"><b>Solitude</b></a> · 11 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/starship.md"><img src="docs/mosaics/starship.jpg" alt="12 Starship wallpapers"></a><br><a href="themes/starship.md"><b>Starship</b></a> · 12 wallpapers</td>
<td width="50%" valign="top"><a href="themes/tokyo-night.md"><img src="docs/mosaics/tokyo-night.jpg" alt="11 Tokyo Night wallpapers"></a><br><a href="themes/tokyo-night.md"><b>Tokyo Night</b></a> · 11 wallpapers</td>
</tr>
<tr>
<td width="50%" valign="top"><a href="themes/vantablack.md"><img src="docs/mosaics/vantablack.jpg" alt="10 Vantablack wallpapers"></a><br><a href="themes/vantablack.md"><b>Vantablack</b></a> · 10 wallpapers</td>
<td width="50%" valign="top"><a href="themes/white.md"><img src="docs/mosaics/white.jpg" alt="13 White wallpapers"></a><br><a href="themes/white.md"><b>White</b></a> · 13 wallpapers</td>
</tr>
</table>
<!-- gallery:end -->

## How the sets were picked

Aether scored each candidate. For every image, Aether built the palette it
would make in the theme's light or dark mode. The score is how far that
palette sits from the theme's own 16 colours. A brightness check kept dark
images out of light themes and the reverse. Near-duplicates, logos, game art
and character art were dropped, and a person made the final picks from
contact sheets.

The paintings come from Wikimedia Commons, at 3840 pixels wide where the
original allows it. Sargent's are all 3000 pixels or wider. Of John
Martin's, ten are 3840 pixels wide and nine are 1920. Most paintings appear in one or two themes. Dracula, Hackerman,
Lumon, Lupine, Retro 82 and Vantablack have none. Commons limits downloads of
full-size files, so the 12 Sargent paintings narrower than 3840 pixels are
stored in [`images/`](images). Cabanel's *Ophelia* had no source large
enough, so this repository stores a 4K restoration made with the SeedVR2
upscaler.

## Credits

The images belong to their creators. Each one links to its source on its
theme page and in the gallery.

| Collection | Wallpapers | Licence |
| --- | --- | --- |
| [dharmx/walls](https://github.com/dharmx/walls) | 125 | None stated |
| [krishna4a6av/Wallpapers](https://github.com/krishna4a6av/Wallpapers) | 35 | None stated |
| [orangci/walls-catppuccin-mocha](https://github.com/orangci/walls-catppuccin-mocha) | 34 | None stated |
| [AngelJumbo/gruvbox-wallpapers](https://github.com/AngelJumbo/gruvbox-wallpapers) | 21 | None stated |
| [rose-pine/wallpapers](https://github.com/rose-pine/wallpapers) | 8 | CC0 |
| [dracula/wallpaper](https://github.com/dracula/wallpaper) | 5 | [MIT](https://github.com/dracula/wallpaper/blob/f2b8cc4223bcc2dfd5f165ab80f701bbb84e3303/LICENSE) |
| [aynp/dracula-wallpapers](https://github.com/aynp/dracula-wallpapers) | 5 | [CC BY-SA 4.0](https://github.com/aynp/dracula-wallpapers/blob/b74123cab099b9fa6d83b5c2041a9030b58f3eb8/LICENSE) |
| [D3Ext/aesthetic-wallpapers](https://github.com/D3Ext/aesthetic-wallpapers) | 4 | MIT |
| [Apeiros-46B/everforest-walls](https://github.com/Apeiros-46B/everforest-walls) | 3 | None stated |
| [John Singer Sargent](https://commons.wikimedia.org/wiki/Category:Paintings_by_John_Singer_Sargent), Wikimedia Commons | 35 | Public domain |
| [John Martin](https://commons.wikimedia.org/wiki/Category:Paintings_by_John_Martin), Wikimedia Commons | 19 | Public domain |
| [Alexandre Cabanel](images), restored here | 1 | Public domain |

Counts are rows, so an image in two themes counts twice. Most community
collections gather art without a stated licence. This repository does not
redistribute those images: the installer downloads each one from its source,
pinned to an exact commit, for personal desktop use. The gallery shows small
previews that link back to the source. If you made one of these images and
want it credited differently or removed, [open an issue](https://github.com/justcarlson/omarchy-wallpapers/issues).

## Contributing

Found a wallpaper that belongs in a set?
[Suggest it](https://github.com/justcarlson/omarchy-wallpapers/issues/new?template=suggest-a-wallpaper.yml)
with a link to the original. The sets are edited upstream and synced here,
so a pull request that edits `wallpapers.tsv` will be redone by hand; an issue
is faster.

To work on the installer or the gallery, see [CONTRIBUTING.md](CONTRIBUTING.md).

The code is [MIT](LICENSE). Made for [Omarchy](https://omarchy.org).
