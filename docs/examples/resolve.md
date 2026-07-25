```bash
# Resolve a full manifest
nap resolve nap://toystory/character/lukeskywalker

# Resolve as JSON
nap resolve nap://toystory/character/lukeskywalker -f json

# Resolve at a specific branch
nap resolve nap://toystory/character/lukeskywalker --branch canon

# Resolve a subtree via fragment query
nap resolve nap://toystory/character/lukeskywalker#properties.species
# → human

# Resolve a nested subtree
nap resolve nap://toystory/character/lukeskywalker#references.appears_in
```
