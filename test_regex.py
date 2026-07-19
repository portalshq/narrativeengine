import re
text = "The universe is big."
pattern = r'\b(universe|universes)\b'
match = re.search(pattern, text, flags=re.IGNORECASE)
print(f"Match: {match}")
