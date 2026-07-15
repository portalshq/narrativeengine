```bash
# Resolve a full manifest
nap resolve nap://starwars/character/lukeskywalker

# Resolve as JSON
nap resolve nap://starwars/character/lukeskywalker -f json

# Resolve at a specific branch
nap resolve nap://starwars/character/lukeskywalker --branch canon

# Resolve a subtree via fragment query
nap resolve nap://starwars/character/lukeskywalker#properties.species
# → human

# Resolve a nested subtree
nap resolve nap://starwars/character/lukeskywalker#references.appears_in
```
