import re
file_path = 'crates/nap-core/src/resolver.rs'
with open(file_path, 'r') as f:
    content = f.read()
new_content = re.sub(r'list_universes\b', 'list_repositories', content, flags=re.IGNORECASE)
if new_content != content:
    print("MATCHES AND MODIFIES")
else:
    print("DOES NOT MODIFY")
