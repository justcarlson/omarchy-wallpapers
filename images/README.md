# Images stored here

Most wallpapers are downloaded from their source. The installer gets the
images in this folder from this repository instead.

## alexandre-cabanel-ophelia.jpg

Alexandre Cabanel, *Ophelia*, 1883. The painting is in the public domain.

No copy online is large enough for a 4K screen. This file is the
[Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Alexandre_Cabanel,_Ophelia.JPG)
image, 1195 pixels wide, upscaled to 3806 × 2456 with
[SeedVR2](https://github.com/numz/ComfyUI-SeedVR2_VideoUpscaler) 7B. A larger
but blurrier 1666-pixel copy gave worse results with every upscaler, so the
smaller, cleaner copy was used. No new rights are claimed in the upscale.

## sargent-*.jpg

John Singer Sargent paintings, all in the public domain, copied unchanged
from Wikimedia Commons. Each is between 3000 and 3811 pixels wide, so Commons
has no 3840-pixel thumbnail of it, and Commons answers repeated downloads of
full-size files with HTTP 429. Each row of `wallpapers.tsv` for these files
names its Commons source in the `origin` column. `gallery sync` stores them
here.
