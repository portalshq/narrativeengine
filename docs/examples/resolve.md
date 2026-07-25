```bash
# Resolve a full manifest
nap resolve nap://toystory/character/woody

# Resolve as JSON
nap resolve nap://toystory/character/woody -f json

# Resolve at a specific branch
nap resolve nap://toystory/character/woody --branch canon

# Resolve a subtree via fragment query
nap resolve nap://toystory/character/woody#properties.toy_type
# → human

# Resolve a nested subtree
nap resolve nap://toystory/character/woody#references.appears_in
```
