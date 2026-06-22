---
title: "1"
sort: 1
category: "macOS reset"
description: "the first file in the macOS reset project"
date: 2026-5-1
tags:
  - macOS
  - reset
  - 1
  - one
  - first
  - initial
---

# 1

## safari

- open `safari`
- [_arfan-vals-list.val.run_](https://arfan-vals-list.val.run)
- [_MARKDOWN-FILES/1/1.1 _ one.md_](https://github.com/imarfanc/warp-251224/blob/main/MARKDOWN-FILES/1/1.1%20_%20one.md)
- get iPhone/iPad
- sign into google

## terminal

- open `terminal`

## macOS desktop

- rm photos widget
- change computer name
  - `open "x-apple.systempreferences:com.apple.SystemProfiler.AboutExtension"`
- activate clipboard history

## terminal script to quit oepn apps (except terminal & finder)

```sh
osascript -e '
tell application "System Events"
  set quitApps to name of every process whose background only is false
end tell
set skipList to {"Finder", "Terminal"}
repeat with appName in quitApps
  if appName is not in skipList then
    try
      tell application appName to quit
    end try
  end if
end repeat
'
```

```sh
osascript -e 'tell application "System Events" to set quitApps to \'
          -e '  name of every process whose background only is false' \
          -e 'set skipList to {"Finder", "Terminal"}' \
          -e 'repeat with appName in quitApps' \
          -e '  if appName is not in skipList then' \
          -e '    try' \
          -e '      tell application appName to quit' \
          -e '    end try' \
          -e '  end if' \
          -e 'end repeat'
```

## auto login

```sh
open "x-apple.systempreferences:com.apple.preferences.users"
```

### check auto login

```sh
sudo defaults read /Library/Preferences/com.apple.loginwindow autoLoginUser
```

```sh
sudo /usr/bin/osascript -e 'tell application "System Events" to log out'
```
