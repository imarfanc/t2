---
title: "terminal-vertical-bar-window"
sort: "1"
icon: "mynaui:one"
category: rare
description: "AppleScript setup for Terminal Vertical Bar cursor, 110×44 window, and Option-as-Meta"
date: 2026-5-1
tags:
  - macOS
  - reset
  - terminal
  - applescript
  - profiles
name: terminal-vertical-bar-window
subcategory: macOS-reset-26.7b
subsubcategory: "2"
visibility: public
---

# Terminal Profile: Cursor and Window Size

This script opens Terminal Settings, goes to **Profiles**, selects the **Vertical Bar** cursor style, sets the window size to **110 columns × 44 rows**, switches to **Keyboard**, and sets **Use Option as Meta** to **Both Option keys**.

It prints short progress messages in Terminal so you can see which action is being attempted. macOS may require Terminal to be enabled in **System Settings → Privacy & Security → Accessibility**.

Save the code as a `.sh` file and run it, or paste the entire block directly into Terminal.

```sh
zsh <<'EOF'

set -u

osascript <<'APPLESCRIPT'
on debugMessage(messageText)
	log "[Terminal setup] " & messageText
end debugMessage

on matchesLabel(uiItem, wantedLabel)
	tell application "System Events"
		try
			set itemText to name of uiItem as text
			if itemText is wantedLabel or itemText ends with wantedLabel then return true
		end try
		try
			set itemText to description of uiItem as text
			if itemText is wantedLabel or itemText ends with wantedLabel then return true
		end try
		try
			set itemText to value of uiItem as text
			if itemText is wantedLabel or itemText ends with wantedLabel then return true
		end try
	end tell
	return false
end matchesLabel

on findControl(rootElement, wantedLabel, allowedRoles)
	tell application "System Events"
		set candidates to entire contents of rootElement
		repeat with candidateReference in candidates
			set candidate to contents of candidateReference
			try
				set candidateRole to role of candidate as text
				if candidateRole is in allowedRoles and my matchesLabel(candidate, wantedLabel) then
					return candidate
				end if
			end try
		end repeat
	end tell
	return missing value
end findControl

on pressControl(controlItem)
	tell application "System Events"
		try
			perform action "AXPress" of controlItem
		on error
			click controlItem
		end try
	end tell
end pressControl

on findTextFieldToRight(rootElement, labelText)
	set labelItem to my findControl(rootElement, labelText, {"AXStaticText"})
	if labelItem is missing value then return missing value

	tell application "System Events"
		set labelPosition to position of labelItem
		set labelX to item 1 of labelPosition
		set labelY to item 2 of labelPosition
		set bestField to missing value
		set bestHorizontalDistance to 100000
		set candidates to entire contents of rootElement

		repeat with candidateReference in candidates
			set candidate to contents of candidateReference
			try
				if (role of candidate as text) is "AXTextField" then
					set fieldPosition to position of candidate
					set fieldX to item 1 of fieldPosition
					set fieldY to item 2 of fieldPosition
					set verticalDistance to fieldY - labelY
					if verticalDistance < 0 then set verticalDistance to -verticalDistance
					set horizontalDistance to fieldX - labelX

					if horizontalDistance > 0 and verticalDistance ≤ 12 and horizontalDistance < bestHorizontalDistance then
						set bestField to candidate
						set bestHorizontalDistance to horizontalDistance
					end if
				end if
			end try
		end repeat
	end tell

	return bestField
end findTextFieldToRight

on findControlToRight(rootElement, labelText, wantedRole)
	set labelItem to my findControl(rootElement, labelText, {"AXStaticText"})
	if labelItem is missing value then return missing value

	tell application "System Events"
		set labelPosition to position of labelItem
		set labelX to item 1 of labelPosition
		set labelY to item 2 of labelPosition
		set bestControl to missing value
		set bestHorizontalDistance to 100000
		set candidates to entire contents of rootElement

		repeat with candidateReference in candidates
			set candidate to contents of candidateReference
			try
				if (role of candidate as text) is wantedRole then
					set controlPosition to position of candidate
					set controlX to item 1 of controlPosition
					set controlY to item 2 of controlPosition
					set verticalDistance to controlY - labelY
					if verticalDistance < 0 then set verticalDistance to -verticalDistance
					set horizontalDistance to controlX - labelX

					if horizontalDistance > 0 and verticalDistance ≤ 12 and horizontalDistance < bestHorizontalDistance then
						set bestControl to candidate
						set bestHorizontalDistance to horizontalDistance
					end if
				end if
			end try
		end repeat
	end tell

	return bestControl
end findControlToRight

on replaceTextFieldValue(textField, newValue, fieldLabel)
	tell application "System Events"
		set focused of textField to true
		keystroke "a" using command down
		keystroke newValue
		key code 48
		delay 0.1

		set resultingValue to value of textField as text
		if resultingValue is not newValue then
			error fieldLabel & " should be " & newValue & ", but its value is " & resultingValue & "."
		end if
	end tell

	my debugMessage(fieldLabel & " set to " & newValue & ".")
end replaceTextFieldValue

my debugMessage("Opening Terminal Settings...")

tell application "Terminal" to activate
tell application "System Events"
	tell process "Terminal"
		set frontmost to true
		keystroke "," using command down
	end tell
end tell

delay 0.35

tell application "System Events"
	tell process "Terminal"
		if not (exists front window) then error "Terminal Settings did not open."
		set settingsWindow to front window
	end tell
end tell

my debugMessage("Opening Profiles...")

set profilesButton to my findControl(settingsWindow, "Profiles", {"AXButton", "AXRadioButton"})
if profilesButton is missing value then error "Could not find the Profiles button."
my pressControl(profilesButton)

delay 0.2

my debugMessage("Opening the Text tab...")

set textTab to my findControl(settingsWindow, "Text", {"AXRadioButton", "AXButton"})
if textTab is missing value then error "Could not find the Text tab."
my pressControl(textTab)

delay 0.15

my debugMessage("Selecting the Vertical Bar cursor style...")

set verticalBarButton to my findControl(settingsWindow, "Vertical Bar", {"AXRadioButton"})
if verticalBarButton is missing value then
	error "Could not find the Vertical Bar cursor radio button in the Text tab."
end if

my pressControl(verticalBarButton)
delay 0.1

tell application "System Events"
	try
		set verticalBarValue to value of verticalBarButton
		my debugMessage("Vertical Bar radio-button value: " & verticalBarValue)
	end try
end tell

my debugMessage("Opening the Window tab...")

set windowTab to my findControl(settingsWindow, "Window", {"AXRadioButton", "AXButton"})
if windowTab is missing value then error "Vertical Bar was selected, but the Window tab could not be found."
my pressControl(windowTab)

delay 0.15

my debugMessage("Setting Columns to 110 and Rows to 44...")

set columnsField to my findTextFieldToRight(settingsWindow, "Columns:")
if columnsField is missing value then error "Could not find the Columns field."
my replaceTextFieldValue(columnsField, "110", "Columns")

set rowsField to my findTextFieldToRight(settingsWindow, "Rows:")
if rowsField is missing value then error "Could not find the Rows field."
my replaceTextFieldValue(rowsField, "44", "Rows")

my debugMessage("Opening the Keyboard tab...")

set keyboardTab to my findControl(settingsWindow, "Keyboard", {"AXRadioButton", "AXButton"})
if keyboardTab is missing value then error "The values were changed, but the Keyboard tab could not be found."
my pressControl(keyboardTab)

delay 0.15

my debugMessage("Setting Use Option as Meta to Both Option keys...")

set optionMetaMenu to my findControlToRight(settingsWindow, "Use Option as Meta:", "AXPopUpButton")
if optionMetaMenu is missing value then error "Could not find the Use Option as Meta menu."

tell application "System Events"
	perform action "AXPress" of optionMetaMenu
	delay 0.15
	perform action "AXPress" of menu item "Both Option keys" of menu 1 of optionMetaMenu
	delay 0.25

	set optionMetaValue to value of optionMetaMenu as text
	if optionMetaValue is not "Both Option keys" then
		error "Use Option as Meta should be Both Option keys, but its value is " & optionMetaValue & "."
	end if
end tell

my debugMessage("Use Option as Meta set to Both Option keys.")
my debugMessage("Done: Vertical Bar -> 110 x 44 -> Keyboard -> Both Option keys")
APPLESCRIPT
EOF
```
