```bash
# Save a generated scene clip as a video representation
nap add nap://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"

# Save another take under a distinct representation key
nap add nap://toystory/scene/pizza-planet clip-02 ./pizza-planet-clip-02.mp4 --format mp4 -m "Add alternate pizza-planet scene clip"

# Inspect the scene and direct representation provenance
nap resolve nap://toystory/scene/pizza-planet --provenance
```
