# notes-3.1

- [x] add 2 new articles
- [x] style headings
- [x] style lists
- add HTML parsing
- add to macOS dictionary article
- [ ] emails & common snippets
    - [ ] link to snippets
- [x] blur after copying / middle click to blur element & undo blur
    - [ ] change to right click
- [ ] try a different code highlighting theme
- [x] copy some styles from daisyui-1 [https://arfan25.val.run/](https://arfan25.val.run/)
- [x] change font in edit modal
- [ ] try a different premade css
- [ ] for task lists , don't show '-' dash
- [ ] and keyboard shortcut to show help
- [ ] center heading 2
- [ ] add toast

```bash
brew install --cask tiny-shield
brew install --cask little-snitch@nightly
brew install --cask font-annotation-mono
brew install --cask voicenotes
brew install --cask hammerspoon
```

```bash
brew update
brew outdated
brew upgrade
brew cleanup
```

```bash
defaults write NSGlobalDomain AppleShowAllFiles -bool true
# defaults write NSGlobalDomain AppleShowAllFiles -bool false
```

```bash
hs -c "require('app_launcher_daisyui').toggleWindow()"
```


```bash
# read global
defaults read NSGlobalDomain
```

---

## md1

``````markdown
Of course! You can view `NSGlobalDomain` settings using the built-in `defaults` command in the macOS terminal.

Here are the most common ways to do it, from the simplest to more specific.

### 1. View All Global Settings

This is the most direct command. `NSGlobalDomain` has a convenient shorthand flag: `-g`.

```bash
defaults read -g
```

Alternatively, you can use the full domain name, which does the exact same thing:

```bash
defaults read NSGlobalDomain
```

**What you'll see:**
This will print a long list of all the global key-value pairs that define system-wide preferences. The output looks like a dictionary or a set of key-value assignments.

Example output might look like this:

```
{
    AppleInterfaceStyle = Dark;
    AppleLanguages =     (
        en-US,
        "zh-Hans-US"
    );
    AppleLocale = "en_US";
    "com.apple.mouse.scaling" = "1.5";
    "com.apple.sound.beep.feedback" = 0;
    ...and many more lines...
}
```

### 2. Search for a Specific Setting

The output from the first command can be very long. If you're looking for something specific, it's best to pipe the output to `grep` to filter it.

For example, to find all settings related to the "Dock":

```bash
# -i makes the search case-insensitive
defaults read -g | grep -i "dock"
```

To find the setting for the interface style (Light/Dark mode):

```bash
defaults read -g | grep AppleInterfaceStyle
```

### 3. Read a Specific Key

If you already know the exact name of the key you want to inspect, this is the most efficient method.

**Syntax:** `defaults read -g <key_name>`

**Examples:**

*   **Check the current interface style (Dark/Light Mode):**
    ```bash
    defaults read -g AppleInterfaceStyle
    ```
    (This will return `Dark` or `Light`, or it might return nothing if it's set to "Auto" and has never been manually overridden).

*   **Check your locale:**
    ```bash
    defaults read -g AppleLocale
    ```

*   **Check the key repeat rate:**
    ```bash
    defaults read -g KeyRepeat
    ```

### Where Is This Data Stored?

While `NSGlobalDomain` is an abstraction, most of these settings are physically stored in a property list (plist) file located in your user's Library folder.

The primary file is: `~/Library/Preferences/.GlobalPreferences.plist`

You can view the contents of this file directly using the `plutil` (property list utility) command, which is useful for seeing the raw data.

```bash
# The -p flag prints the plist in a human-readable, old-style format
plutil -p ~/Library/Preferences/.GlobalPreferences.plist
```

This will produce output very similar to `defaults read -g`, because the `defaults` command is essentially the standard tool for reading from and writing to this file and other preference files.

### Summary / Cheatsheet

| Task                                 | Command                                                    |
| ------------------------------------ | ---------------------------------------------------------- |
| **View all** global settings         | `defaults read -g`                                         |
| **Search for** a setting (e.g., "scroll") | `defaults read -g \| grep -i "scroll"`                     |
| **Read one specific** key          | `defaults read -g AppleInterfaceStyle`                     |
| **View the underlying file**       | `plutil -p ~/Library/Preferences/.GlobalPreferences.plist` |

---

**A friendly warning:** The `defaults` command can also `write` and `delete` keys. Be very careful with these commands unless you know exactly what you are doing, as changing a global setting can affect the behavior of many applications and the system itself. Always use `read` to inspect things safely.

``````

---

## md2

``````md
Excellent question! You can do that by redirecting the output of the command to a file.

Here is the exact command you need:

```bash
defaults read NSGlobalDomain > ~/Desktop/1-NSGlobalDomain.txt
```

### How to Use It

1.  Open your **Terminal** application.
2.  Copy the command above.
3.  Paste it into the Terminal window.
4.  Press **Enter**.

You won't see any output in the terminal window itself. Instead, the command will run silently, and a new file named `1-NSGlobalDomain.txt` will appear on your Desktop containing all of your `NSGlobalDomain` settings.

### Breakdown of the Command

Let's break down what each part does:

*   `defaults read NSGlobalDomain`
    *   This is the command you already know, which reads all the global preferences and prints them to "standard output" (by default, your terminal screen).

*   `>`
    *   This is the **output redirection operator**. It tells the shell to take whatever output the command on the left produces and "redirect" it into the file specified on the right.
    *   **Important:** If the file `1-NSGlobalDomain.txt` already exists, this `>` operator will **overwrite** it completely.

*   `~/Desktop/1-NSGlobalDomain.txt`
    *   This is the full path to your destination file.
    *   `~` is a shortcut for your user's home directory (e.g., `/Users/yourusername`).
    *   `/Desktop/` is the Desktop folder inside your home directory.
    *   `1-NSGlobalDomain.txt` is the name of the file you want to create.

---

### Alternative: Appending to a File

If you wanted to **add** the output to the end of a file without overwriting it, you would use two greater-than signs (`>>`) instead.

For example, to first save the current date and time to the file, and *then* append the settings:

```bash
# This creates the file and puts the date in it
date > ~/Desktop/1-NSGlobalDomain.txt

# This appends the settings to the same file on a new line
defaults read NSGlobalDomain >> ~/Desktop/1-NSGlobalDomain.txt
```

---

Of course! To include the current date in the filename, you can use **command substitution**. This lets you run a command (like `date`) and use its output as part of another command.

Here is the command to use. This example will create a filename like `NSGlobalDomain-2023-10-27.txt`.

### Recommended Command (YYYY-MM-DD Format)

```bash
defaults read NSGlobalDomain > ~/Desktop/NSGlobalDomain-$(date +%Y-%m-%d).txt
```

### How it Works

1.  `date +%Y-%m-%d`
    *   This command gets the current date and formats it.
    *   `%Y` is the full 4-digit year (e.g., 2023).
    *   `%m` is the 2-digit month (e.g., 10).
    *   `%d` is the 2-digit day (e.g., 27).

2.  `$(...)`
    *   This is the **command substitution** syntax. The shell first runs the command inside the parentheses (`date ...`), gets its output (e.g., "2023-10-27"), and then substitutes it into the main command line.

3.  The final command that the shell executes becomes (for today):
    `defaults read NSGlobalDomain > ~/Desktop/NSGlobalDomain-2023-10-27.txt`

---

### Other Formatting Options

You can customize the date and time format however you like. Here are a few popular examples.

#### To include the time (for more precise snapshots):

This creates a filename like `NSGlobalDomain-2023-10-27_15-30-55.txt`.

```bash
# Format: YYYY-MM-DD_HH-MM-SS
defaults read NSGlobalDomain > ~/Desktop/NSGlobalDomain-$(date +%Y-%m-%d_%H-%M-%S).txt
```

#### For a more compact, sortable filename:

This creates a filename like `20231027-NSGlobalDomain.txt`, which is great for sorting files by name.

```bash
# Format: YYYYMMDD at the beginning
defaults read NSGlobalDomain > ~/Desktop/$(date +%Y%m%d)-NSGlobalDomain.txt
```

Just copy and paste the version you prefer into your terminal and press **Enter**. A new, date-stamped file will be created on your Desktop.

``````

---

## md3

``````md
Excellent idea. Putting the date at the beginning is great for sorting files chronologically.

To match the format `YY.M.D`, you will use specific codes with the `date` command.

Here is the command you're looking for:

```bash
defaults read NSGlobalDomain > ~/Desktop/$(date +%y.%m.%d)-NSGlobalDomain.txt
```

### How to Use It

1.  Copy the command above.
2.  Paste it into your Terminal.
3.  Press **Enter**.

On October 27, 2023, this will create a file on your Desktop named: `23.10.27-NSGlobalDomain.txt`.

### Breakdown of the Date Format

*   `date +%y.%m.%d`
    *   `%y` is the **2-digit year** (e.g., 23).
    *   `%m` is the **2-digit month** (e.g., 01 for January, 10 for October).
    *   `%d` is the **2-digit day** of the month (e.g., 05 for the 5th).
    *   The `.` characters are treated as literal separators.

---

### A Note on Single Digits (M vs. %m)

You requested `M.D`, which can sometimes imply a single digit for months and days (e.g., `23.1.5` for January 5th).

The standard `date` command on macOS doesn't have a simple, direct flag for a non-padded month or day. Using the two-digit, zero-padded format (`%m` and `%d`) is actually the recommended practice for a few key reasons:

1.  **Correct Sorting:** Files will sort correctly chronologically when ordered by name.
    *   **Good:** `23.10.27` comes after `23.01.05`.
    *   **Bad:** `23.1.5` might sort *after* `23.10.27` in some file systems because the string "10" comes before "5" alphabetically.

2.  **Consistency:** The filenames will always have the same length, making them look cleaner in a list.

For these reasons, the command provided above using `%y.%m.%d` is the most robust and practical way to achieve your goal.

---

Got it. That's a great formatting choice, but it's a bit trickier because the standard `date` command on macOS doesn't have a simple flag for single-digit months or days (like `%e` or `%-d` which exist in other versions of Unix/Linux).

No problem! We can achieve this by telling the shell to treat the month and day as numbers, which automatically removes any leading zeros.

Here are two ways to do it: a clear step-by-step version and a compact one-liner.

### Method 1: The Clear, Step-by-Step Version

This is the easiest to read and understand. You can paste this whole block into your terminal.

```bash
# Get the parts of the date and store them in variables
year=$(date +%y)
month=$((10#$(date +%m))) # Treat month as a base-10 number
day=$((10#$(date +%d)))   # Treat day as a base-10 number

# Use the variables to build the filename and save the output
defaults read NSGlobalDomain > ~/Desktop/"$year.$month.$day-NSGlobalDomain.txt"
```

### Method 2: The Compact One-Liner

This combines all the logic into a single line. It's faster to copy-paste but harder to read.

```bash
defaults read NSGlobalDomain > ~/Desktop/"$(date +%y).$((10#$(date +%m))).$((10#$(date +%d)))-NSGlobalDomain.txt"
```

### How It Works

*   `$(date +%y)`: This gets the **2-digit year** as before (e.g., `23`).
*   `$((...))`: This is called **arithmetic expansion**. It tells the shell to calculate the result of the expression inside.
*   `10#$(date +%m)`: This is the key part.
    *   `$(date +%m)` gets the zero-padded month (e.g., `09` for September).
    *   The `10#` prefix forces the shell to interpret `09` as a **base-10 (decimal) number**. When it does this, the leading `0` is dropped, resulting in `9`. This also correctly handles `10`, `11`, and `12`.
*   The same logic applies to `$((10#$(date +%d)))` for the day.

**Example:**
If you run this on September 5, 2023, either command will create a file on your Desktop named:

`23.9.5-NSGlobalDomain.txt`

``````

---

# md4

``````md
Excellent! Here are the commands for the other domains, using the exact same dynamic date format you liked.

Just copy and paste each one into your terminal and press Enter.

---

### 1. For Finder Settings (`com.apple.finder`)

This will save your Finder preferences to a file like `23.9.5-com.apple.finder.txt`.

```bash
defaults read com.apple.finder > ~/Desktop/"$(date +%y).$((10#$(date +%m))).$((10#$(date +%d)))-com.apple.finder.txt"
```

### 2. For Dock Settings (`com.apple.dock`)

This will save your Dock preferences to a file like `23.9.5-com.apple.dock.txt`.

```bash
defaults read com.apple.dock > ~/Desktop/"$(date +%y).$((10#$(date +%m))).$((10#$(date +%d)))-com.apple.dock.txt"
```

### 3. For Universal Access Settings (`com.apple.universalaccess`)

This will save your Accessibility/Universal Access preferences to a file like `23.9.5-com.apple.universalaccess.txt`.

```bash
defaults read com.apple.universalaccess > ~/Desktop/"$(date +%y).$((10#$(date +%m))).$((10#$(date +%d)))-com.apple.universalaccess.txt"
```

Each command will create a separate, date-stamped text file on your Desktop containing the settings for that specific domain.

---

### Pro Tip: Run All at Once with a Loop

Instead of copying and pasting three separate commands, you can use a simple `for` loop to do it all in one go. This is a very powerful technique for automating tasks.

Just copy this entire block, paste it into your terminal, and press Enter. It will create all three files for you automatically.

```bash
# Define the date part once to be efficient
thedate="$(date +%y).$((10#$(date +%m))).$((10#$(date +%d)))"

# List the domains you want to export
for domain in com.apple.finder com.apple.dock com.apple.universalaccess
do
  echo "Exporting settings for $domain..."
  defaults read "$domain" > ~/Desktop/"$thedate-$domain.txt"
done

echo "All done!"
```

**How the loop works:**
1.  It lists the three domains you want.
2.  It loops through them, one by one, temporarily assigning each name to the variable `$domain`.
3.  Inside the loop, it runs the `defaults read` command, using the `$domain` variable for both the domain to read and the output filename.
4.  The `echo` commands just print progress messages to your terminal so you can see what's happening.

``````
