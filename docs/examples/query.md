```bash
# Query a subtree
px query px://toystory/character/woody properties

# Query nested properties
px query px://toystory/character/woody properties.toy_type

# Query as YAML
px query px://toystory/character/woody properties -f yaml
```
