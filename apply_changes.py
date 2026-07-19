import re
import os
import subprocess

def replacer(match):
    word = match.group(0)
    if word.lower() == "universe":
        if word.istitle(): return "Repository"
        if word.isupper(): return "REPOSITORY"
        return "repository"
    if word.lower() == "universes":
        if word.istitle(): return "Repositories"
        if word.isupper(): return "REPOSITORIES"
        return "repositories"
    return word

def process(file_path):
    if not os.path.isfile(file_path): return False
    with open(file_path, 'r', errors='ignore') as f:
        content = f.read()
    
    new_content = re.sub(r'\b(universe|universes)\b', replacer, content, flags=re.IGNORECASE)
    new_content = re.sub(r':universe\b', ':repository', new_content, flags=re.IGNORECASE)
    new_content = re.sub(r'list_universes\b', 'list_repositories', new_content, flags=re.IGNORECASE)
    new_content = re.sub(r'listUniverse\b', 'listRepository', new_content, flags=re.IGNORECASE)
    new_content = re.sub(r'listUniverses\b', 'listRepositories', new_content, flags=re.IGNORECASE)
    
    if new_content != content:
        with open(file_path, 'w') as f:
            f.write(new_content)
        print(f"Modified: {file_path}")
        return True
    return False

# Find files
files_cmd = "git grep -l -i 'universe' -- . ':!node_modules/*' ':!dist/*' ':!docs/generated/*' ':!.*' | grep -v '/dist/'"
files = subprocess.check_output(files_cmd, shell=True).decode().splitlines()

for f in files:
    process(f)
