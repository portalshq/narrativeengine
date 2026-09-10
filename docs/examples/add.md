```bash
# Save a generated scene clip as a video representation
px add px://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"

# Save another take under a distinct representation key
px add px://toystory/scene/pizza-planet clip-02 ./pizza-planet-clip-02.mp4 --format mp4 -m "Add alternate pizza-planet scene clip"

# Inspect the scene and direct representation provenance
px resolve px://toystory/scene/pizza-planet --provenance
```
