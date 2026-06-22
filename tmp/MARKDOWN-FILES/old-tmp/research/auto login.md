# auto login

## gpt-5-mini

### check auto login

Run this in Terminal to check Auto Login status (macOS):

`sudo defaults read /Library/Preferences/com.apple.loginwindow autoLoginUser`

- If it prints a short username, Auto Login is enabled for that account.
- If it returns "find: plist not found" or an error, Auto Login is not set.
- You may also get "The domain/default pair..." if key absent — meaning disabled.

(You may be prompted for your admin password.)

### open settings

On macOS 14 (Sonoma) and later Apple moved to the x-apple.systempreferences URL scheme, but the exact identifier can vary by macOS release. For macOS 13 (Ventura) and newer the correct Users & Groups URL is usually:

`open "x-apple.systempreferences:com.apple.preferences.users"`

If that doesn't work, use the macOS Monterey and earlier (System Preferences):

`open /System/Library/PreferencePanes/Accounts.prefPane`

You can also list available preference pane URL identifiers with:

`ls /System/Library/PreferencePanes`

### logout

To log out immediately (current user):

`sudo /usr/bin/osascript -e 'tell application "System Events" to log out'`

To shut down now:

`sudo shutdown -h now`

Alternative shut down with a 1‑minute warning:

`sudo shutdown -h +1 "System will shut down in 1 minute"`

Force immediate shutdown (no clean logout):

`sudo halt`

You’ll be prompted for your admin password when using sudo.

## system settings
