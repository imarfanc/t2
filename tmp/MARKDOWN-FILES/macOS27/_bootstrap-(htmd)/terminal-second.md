---
title: "terminal-second"
sort: "4"
category: rare
description: "Initial file and directory cleanup script"
date: 2026-5-1
tags:
  - macOS
  - reset
  - cleanup
  - terminal
name: terminal-second
icon: "glyphs-poly:four"
subcategory: macOS-reset-26.7b
subsubcategory: _bootstrap
visibility: public
---

# Terminal Second Setup

## Delete Files & Directories

```sh
bash <<'EOF'
echo "deleting files & dirs..."
file_count=0
dir_count=0
while IFS= read -r -d '' f; do
  if [ -d "$f" ]; then
    dir_count=$((dir_count + 1))
    printf "\r  deleting dir [%d] %s                    " "$dir_count" "$f"
  else
    file_count=$((file_count + 1))
    printf "\r  deleting file [%d] %s                    " "$file_count" "$f"
  fi
  sudo rm -rf -- "$f" 2>/dev/null
done < <(sudo find ~/Movies ~/Music ~/Public ~/Downloads ~/Desktop ~/Documents \
  -mindepth 1 -print0 2>/dev/null)
printf "\r  deleted %d dirs, %d files                    \n" "$dir_count" "$file_count"

echo "deleting .DS_Store..."
ds_count=0
while IFS= read -r -d '' f; do
  ds_count=$((ds_count + 1))
  printf "\r  deleting [%d] %s                    " "$ds_count" "$f"
  sudo rm -f -- "$f" 2>/dev/null
done < <(sudo find ~ \
  -name ".DS_Store" -type f -print0 2>/dev/null)
printf "\r  deleted %d .DS_Store files                    \n" "$ds_count"
EOF
```
