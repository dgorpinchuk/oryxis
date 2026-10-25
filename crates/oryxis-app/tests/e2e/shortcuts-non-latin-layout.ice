viewport: 1200x750
mode: Zen
-----
# A chord under a layout that types another script (PR #270). `key` on
# the event is the character the layout types (under Russian the key
# labelled C reads `с`), and the app resolves a chord by the US layout's
# character at the key's position when the typed one is outside ASCII
# (hotkeys::chord_key). `с@keyc` names both halves of such a press. Two
# doors are proved:
# a factory chord fires when typed as Cyrillic (Ctrl+, opens Settings,
# the comma key types `б`), and the Shortcuts capture records the Latin
# letter for a chord typed as Cyrillic (the key labelled X types `ч`;
# no factory chord carries X, and `order` matches a text exactly, so the
# X chip sits once in the table, on the row that recorded it).
expect "Welcome to Oryxis"
click "Skip"
click "Continue without password"
settle 250
expect "Hosts"
type ctrl+б@comma
settle 300
expect "Shortcuts"
click "Shortcuts"
settle
expect "Reconnect"
# The Ctrl+Shift+R chip of the "Reconnect" row (the last row the
# 750 px viewport shows whole).
click (270, 697)
settle 250
expect "Press a key or mouse button…"
type ctrl+alt+ч@keyx
settle 250
absent "Press a key or mouse button"
order "X" "Reconnect"
