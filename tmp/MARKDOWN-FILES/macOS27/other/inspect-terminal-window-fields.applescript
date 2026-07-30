on safeText(theValue)
	try
		if theValue is missing value then return "<missing>"
		return theValue as text
	on error
		return "<?>"
	end try
end safeText

tell application "System Events"
	tell process "Terminal"
		set settingsWindow to front window
		set initialItems to entire contents of settingsWindow
		repeat with itemReference in initialItems
			set uiItem to contents of itemReference
			try
				if (role of uiItem as text) is "AXRadioButton" and (name of uiItem as text) is "Window" then
					perform action "AXPress" of uiItem
					exit repeat
				end if
			end try
		end repeat
		delay 0.2
		set allItems to entire contents of settingsWindow
		set outputText to "window=" & my safeText(name of settingsWindow) & linefeed
		set itemNumber to 0
		repeat with itemReference in allItems
			set itemNumber to itemNumber + 1
			set uiItem to contents of itemReference
			try
				set itemRole to role of uiItem as text
				if itemRole is in {"AXTextField", "AXStaticText", "AXRadioButton", "AXIncrementor"} then
					set itemName to "<missing>"
					set itemDescription to "<missing>"
					set itemValue to "<missing>"
					set itemPosition to "<missing>"
					set itemSize to "<missing>"
					try
						set itemName to my safeText(name of uiItem)
					end try
					try
						set itemDescription to my safeText(description of uiItem)
					end try
					try
						set itemValue to my safeText(value of uiItem)
					end try
					try
						set itemPosition to my safeText(position of uiItem)
					end try
					try
						set itemSize to my safeText(size of uiItem)
					end try
					set outputText to outputText & itemNumber & " | " & itemRole & " | name=" & itemName & " | desc=" & itemDescription & " | value=" & itemValue & " | pos=" & itemPosition & " | size=" & itemSize & linefeed
				end if
			end try
		end repeat
		return outputText
	end tell
end tell
