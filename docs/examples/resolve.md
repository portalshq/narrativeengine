```bash
# Resolve a full manifest
px resolve px://toystory/character/woody

# Resolve as JSON
px resolve px://toystory/character/woody -f json

# Resolve at a specific branch
px resolve px://toystory/character/woody --branch canon

# Resolve a subtree via fragment query
px resolve px://toystory/character/woody#properties.toy_type
# → human

# Resolve a nested subtree
px resolve px://toystory/character/woody#references.appears_in
```
