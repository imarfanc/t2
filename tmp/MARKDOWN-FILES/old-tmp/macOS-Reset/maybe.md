# to be added if

## quarantine

```sh
xattr -dr com.apple.quarantine /Applications/WinMux.app/
```

## failed at gws auth

```sh
brew install --cask google-cloud-sdk
npm install -g @googleworkspace/cli

gws auth setup     # walks you through Google Cloud project config
gws auth login     # subsequent OAuth login
gws drive files list --params '{"pageSize": 5}'
```

## spaced

- https://sindresorhus.com/spaced#older-versions
- https://neobrowser.ai/?utm_source=youtube&utm_medium=influencer&utm_campaign=intheworldofai2
- https://dupeguru.voltaicideas.net/
- https://www.pastebar.app/

# new casks

```sh
brew install pake
brew install --cask handbrake-app
brew install --cask pearcleaner
brew install --cask utm@beta
brew install --cask virtualbuddy
brew install --cask windsurf@next
brew install --cask marta
brew install --cask osaurus
brew install --cask zettlr
brew install --cask raycast
brew install --cask iterm2@beta
brew install --cask localsend
brew install --cask min
# brew tap superset-sh/superset
brew install --cask superset
```

# common terminal commands

```sh
cd ~/Developer/gh/vt
```

# linux cli tools

```md
# Computer Use CLI Tools

## Screen Capture
- **`scrot`** or **`gnome-screenshot`** \u2014 take screenshots
- **`imagemagick`** (`import` command) \u2014 alternative screenshot tool

## Mouse & Keyboard Control
- **`xdotool`** \u2014 simulate mouse movement, clicks, and keyboard input (X11)
- **`ydotool`** \u2014 Wayland alternative to xdotool

## Display Server
- **X11** (most common) or **Wayland**
- **`xvfb`** \u2014 virtual framebuffer for headless environments

## Optional but Useful
- **`tesseract-ocr`** \u2014 OCR to read text from screenshots
- **`python3` + `pyautogui`** \u2014 Python library for screen automation
- **`python3` + `Pillow`** \u2014 image processing

## Quick Install (Debian/Ubuntu)
```bash
sudo apt install scrot xdotool imagemagick xvfb tesseract-ocr
uv pip install pyautogui pillow
```
```
