# article-4.2

## testing

testing

---

Understood. Here's the `awk` command, stripped of comments, and ready to be saved into `Downloads/test/ls_recursive_indented.sh`. I'll wrap it in a simple shell script to make it executable.

First, create the directory if it doesn't exist:

```bash
mkdir -p ~/Downloads/test/
```

Now, create the file `ls_recursive_indented.sh` with the following content:

```bash
#!/bin/bash

ls -R | awk '
  BEGIN { max_depth = 2; current_dir_path = "."; current_dir_level = 0 }
  /:$/ {
    gsub(":","")
    current_dir_path = $0
    if (current_dir_path == ".") {
      current_dir_level = 0
    } else {
      current_dir_level = split(current_dir_path, parts, "/")
    }
    if (current_dir_level < max_depth) {
      indent = ""
      for (i=0; i<current_dir_level; i++) indent = indent "    "
      print indent "├── " current_dir_path "/"
      skip_lines = 0
    } else {
      skip_lines = 1
    }
    next
  }
  /^[[:space:]]*$/ { next }
  {
    if (skip_lines) {
      next
    }
    item_level = current_dir_level + 1
    if (item_level <= max_depth) {
      indent = ""
      for (i=0; i<item_level; i++) indent = indent "    "
      print indent "├── " $0
    }
  }
'
```

To save this directly from your terminal, you can use the `cat` command with a "here document":

```bash
mkdir -p ~/Downloads/test/
cat << 'EOF' > ~/Downloads/test/ls_recursive_indented.sh
#!/bin/bash

ls -R | awk '
  BEGIN { max_depth = 2; current_dir_path = "."; current_dir_level = 0 }
  /:$/ {
    gsub(":","")
    current_dir_path = $0
    if (current_dir_path == ".") {
      current_dir_level = 0
    } else {
      current_dir_level = split(current_dir_path, parts, "/")
    }
    if (current_dir_level < max_depth) {
      indent = ""
      for (i=0; i<current_dir_level; i++) indent = indent "    "
      print indent "├── " current_dir_path "/"
      skip_lines = 0
    } else {
      skip_lines = 1
    }
    next
  }
  /^[[:space:]]*$/ { next }
  {
    if (skip_lines) {
      next
    }
    item_level = current_dir_level + 1
    if (item_level <= max_depth) {
      indent = ""
      for (i=0; i<item_level; i++) indent = indent "    "
      print indent "├── " $0
    }
  }
'
EOF
```

After saving, you need to make the script executable:

```bash
chmod +x ~/Downloads/test/ls_recursive_indented.sh
```

Now you can run it from any directory to see the 2-level deep, indented listing:

```bash
~/Downloads/test/ls_recursive_indented.sh
```

---
---

Okay, to add one more level deep (making it 3 levels in total) and include all files and folders (including hidden ones), we need to make two changes to the `awk` script and one change to the `ls` command:

1.  **Change `max_depth` to `3`**: This will allow `awk` to process and display items up to three levels deep.
2.  **Change `ls -R` to `ls -aR`**: The `-a` option for `ls` means "all", which includes entries starting with `.` (hidden files and directories).

Here's the updated command to generate the output and save it to `~/Downloads/test/ls_output_3levels_all.txt`:

```bash
mkdir -p ~/Downloads/test/
ls -aR | awk '
  BEGIN { max_depth = 3; current_dir_path = "."; current_dir_level = 0 }
  /:$/ {
    gsub(":","")
    current_dir_path = $0
    if (current_dir_path == ".") {
      current_dir_level = 0
    } else {
      current_dir_level = split(current_dir_path, parts, "/")
    }
    if (current_dir_level < max_depth) {
      indent = ""
      for (i=0; i<current_dir_level; i++) indent = indent "    "
      print indent "├── " current_dir_path "/"
      skip_lines = 0
    } else {
      skip_lines = 1
    }
    next
  }
  /^[[:space:]]*$/ { next }
  {
    if (skip_lines) {
      next
    }
    item_level = current_dir_level + 1
    if (item_level <= max_depth) {
      indent = ""
      for (i=0; i<item_level; i++) indent = indent "    "
      print indent "├── " $0
    }
  }
' > ~/Downloads/test/ls_output_3levels_all.txt
```

**Key changes highlighted:**

*   `ls -aR`: This will pass hidden files and directories to `awk`.
*   `BEGIN { max_depth = 3; ... }`: The `max_depth` variable is now set to `3`.

After running this, the file `~/Downloads/test/ls_output_3levels_all.txt` will contain the full listing, including hidden items, up to three levels deep.

---
