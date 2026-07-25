```bash
# Save a generated scene clip as a video representation
nap add nap://toystory/scene/cantina clip-01 ./cantina-clip-01.mp4 --format mp4 -m "Add cantina scene clip"

# Save another take under a distinct representation key
nap add nap://toystory/scene/cantina clip-02 ./cantina-clip-02.mp4 --format mp4 -m "Add alternate cantina scene clip"

# Inspect the scene and direct representation provenance
nap resolve nap://toystory/scene/cantina --provenance
```
