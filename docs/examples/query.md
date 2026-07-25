```bash
# Query a subtree
nap query nap://toystory/character/woody properties

# Query nested properties
nap query nap://toystory/character/woody properties.toy_type

# Query as YAML
nap query nap://toystory/character/woody properties -f yaml
```
