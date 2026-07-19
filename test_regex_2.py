import re
text = "list_universes"
# match: word_boundary + list + _ + universes + word_boundary
match = re.search(r'list_universes\b', text)
print(f"Match: {match}")
