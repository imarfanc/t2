# shell-2.2

## install brew and apps using brew

### 1. install brew
[Homebrew](https://brew.sh/)

```bash
echo -e "ETA: ~4 mins."
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

add to .zshrc

```bash

echo >> /Users/arfan/.zprofile
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> /Users/arfan/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

### 2. check tools & versions

```bash
brew update
brew outdated
brew cleanup
brew upgrade
brew cleanup
brew --version
brew doctor
brew list -1
lsd -1a /Applications
```

### 3. install warp & keyboard maestro

[warp](https://www.warp.dev/)

[keyboard maestro](https://www.keyboardmaestro.com/)

```bash
brew install --cask warp
brew install --cask keyboard-maestro
brew install --cask whatsapp@beta
brew install --cask chromium
brew install --cask shortcat
brew install --cask setapp
# brew install --cask hyperkey
brew install --cask hammerspoon
brew install --cask font-hack-nerd-font
brew install --cask lunar
brew install --cask zed
brew install --cask zen
brew install --cask github@beta
```

### open warp & keyboard-maestro

```bash
open "https://www.dropbox.com/scl/fo/s4elylca2eiqakkxt4lic/ACRa33t1pm5btj7aSZMvmPY?rlkey=d5xcxwkrjqvmhy63dadwz8s1x&st=7cvq9c0i&dl=0"
# hide all Apps except Terminal
osascript -e 'tell application "System Events" to set visible of every process whose name is not "Terminal" to false'
open -a 'Warp'
open -a 'Keyboard Maestro'
```

#### public dropbox link

[cgshaq public dropbox folder](https://www.dropbox.com/scl/fo/s4elylca2eiqakkxt4lic/ACRa33t1pm5btj7aSZMvmPY?rlkey=d5xcxwkrjqvmhy63dadwz8s1x&st=7cvq9c0i&dl=0)
[cgshaq public onedrive fodler](https://1drv.ms/f/c/6d999f1d0e6b4924/Er2Jyfi5wUtGnJHcA8o86WABDFCkYVl9-L76rs2v8gXuFw?e=VhBiNN)

## Add keyboard maestro macro

### add macro

### macro group name
```md test.md
url_macros
```
### macro name
```md test.md
web_2_terminal
```
### macro actions
```xml test.xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
	<dict>
		<key>ActionColor</key>
		<string>Aqua</string>
		<key>ActionUID</key>
		<integer>424</integer>
		<key>AllWindows</key>
		<true/>
		<key>AlreadyActivatedActionType</key>
		<string>Normal</string>
		<key>Application</key>
		<dict>
			<key>BundleIdentifier</key>
			<string>dev.warp.Warp-Stable</string>
			<key>Name</key>
			<string>Warp</string>
			<key>NewFile</key>
			<string>/Applications/Warp.app</string>
		</dict>
		<key>MacroActionType</key>
		<string>ActivateApplication</string>
		<key>ReopenWindows</key>
		<false/>
		<key>TimeOutAbortsMacro</key>
		<true/>
	</dict>
	<dict>
		<key>ActionColor</key>
		<string>Yellow</string>
		<key>ActionUID</key>
		<integer>814</integer>
		<key>MacroActionType</key>
		<string>Pause</string>
		<key>Time</key>
		<string>.8</string>
		<key>TimeOutAbortsMacro</key>
		<true/>
	</dict>
	<dict>
		<key>ActionColor</key>
		<string>Purple</string>
		<key>ActionUID</key>
		<integer>816</integer>
		<key>KeyCode</key>
		<integer>9</integer>
		<key>MacroActionType</key>
		<string>SimulateKeystroke</string>
		<key>Modifiers</key>
		<integer>256</integer>
		<key>ReleaseAll</key>
		<false/>
		<key>TargetApplication</key>
		<dict/>
		<key>TargetingType</key>
		<string>Front</string>
	</dict>
	<dict>
		<key>ActionColor</key>
		<string>Purple</string>
		<key>ActionUID</key>
		<integer>821</integer>
		<key>IsActive</key>
		<false/>
		<key>KeyCode</key>
		<integer>51</integer>
		<key>MacroActionType</key>
		<string>SimulateKeystroke</string>
		<key>Modifiers</key>
		<integer>0</integer>
		<key>ReleaseAll</key>
		<false/>
		<key>TargetApplication</key>
		<dict/>
		<key>TargetingType</key>
		<string>Front</string>
	</dict>
</array>
</plist>
```

### 4. install some apps

```bash
brew install --cask chromium
brew install --cask shortcat
brew install --cask hyperkey
brew install --cask voiceink
brew install --cask hammerspoon
brew install --cask font-hack-nerd-font
brew install --cask zed
brew install --cask zen
```

```bash
sudo xattr -rd com.apple.quarantine /Applications/Chromium.app
open -a 'Chromium'
```

## install brew packages

### 1. install tools using brew

```bash
brew install lsd                        # ls alternative           # 
# brew install eza                       # ls alternative           # 
brew install tree                       # ls alternative           # 
brew install yazi                       # finder alternative       # 
brew install bat                        # cat alternative          # 
brew install fd                         # find alternative          # 
brew install ripgrep                    # grep alternative          # use rg
brew install fzf                        # fuzzy finder              # 
brew install zoxide                     # cd alternative            # 
brew install z                          # cd alternative            # 
eval "$(zoxide init bash)"      # use z
brew install zsh-autosuggestions        # zsh autosuggestions        # 
brew install zsh-syntax-highlighting    # zsh syntax highlighting    # 
brew install zsh-autocomplete           # zsh autocomplete           # 
```

### 2. install more tools using brew

```bash
brew install btop                       # 
brew install htop                       # 
brew install mactop                     # 
brew install neofetch                   # 
# brew install screenfetch                # use neofetch
# brew install screenresolution           # not working
```

### 3. install first* batch using brew

```bash
brew install --cask google-chrome         # chrome           # 
brew install --cask google-chrome@beta    # chrome beta      # 
brew install --cask google-chrome@dev     # chrome dev       # 
brew install --cask google-chrome@canary  # chrome canary    # 
#brew install --cask firefox                  # firefox       # conflicts w/ firefox-beta
brew install --cask firefox@beta
brew install --cask firefox@developer-edition # firefox DE   # 
brew install --cask firefox@nightly
brew install --cask brave-browser            # brave         # 
brew install --cask brave-browser@beta
brew install --cask brave-browser@nightly
lsd /Applications
```

### install browsers using brew

```bash
brew install --cask microsoft-edge        # edge             # 
brew install --cask arc                   # arc              # 
brew install --cask thebrowsercompany-dia # dia   # 
lsd /Applications
```

### install more apps using brew

```bash
brew install --cask ollama
brew install --cask chatgpt
brew install --cask visual-studio-code
brew install --cask visual-studio-code@insiders
brew install --cask cursor
brew install --cask void
brew install --cask trae
brew install --cask windsurf
brew install --cask phoenix-code
lsd /Applications
```

### install other apps using brew

```bash
brew install --cask obsidian
brew install --cask notion
brew install --cask raycast
brew install --cask utm
brew install --cask flux-app
lsd /Applications
```
