```bash
# Query a subtree
nap query nap://starwars/character/lukeskywalker properties

# Query nested properties
nap query nap://starwars/character/lukeskywalker properties.species

# Query as YAML
nap query nap://starwars/character/lukeskywalker properties -f yaml
```
