---
name: first-boot
sort: "1"
icon: "mynaui:one"
category: rare
subcategory: macOS-reset-26.7b
subsubcategory: _bootstrap
description: "Initial desktop and system configuration on macOS first boot"
date: 2026-5-1
tags:
  - macOS
  - reset
  - bootstrap
  - initial
visibility: public
---

# First Boot Setup

## Safari
- Open `Safari`
- [ ] Get iPhone/iPad & sign into Google

## macOS Desktop
- remove photos widget
- remove weather widget

- open `Terminal`

  - Change computer name:
    ```sh
    open "x-apple.systempreferences:com.apple.SystemProfiler.AboutExtension"
    ```
- Activate clipboard history

## Auto Login

```sh
open "x-apple.systempreferences:com.apple.preferences.users"
```

## Terminal Script to Quit Open Apps

- (except Terminal & Finder)

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

### Check Auto Login

- will need `sudo pass`

```sh
sudo defaults read /Library/Preferences/com.apple.loginwindow autoLoginUser
```

```sh
sudo /usr/bin/osascript -e 'tell application "System Events" to log out'
```
