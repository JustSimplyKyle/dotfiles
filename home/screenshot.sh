#!/usr/bin/env bash
if [ "$XDG_CURRENT_DESKTOP" = "sway" ] ; then
	grim -g "$(swaymsg -t get_tree | jq -j '.. | select(.type?) | select(.focused).rect | "\(.x),\(.y) \(.width)x\(.height)"')" - | wl-copy
fi
if [ "$XDG_CURRENT_DESKTOP" = "hyprland" ] ; then
	wayshot -s "$(hyprctl activewindow | sed -n '2p' | sed -e "s/[^0-9]/ /g" | sed -e 's/^[[:space:]]*//') $(hyprctl activewindow | sed -n '3p' | sed -e "s/[^0-9]/ /g" | sed -e 's/^[[:space:]]*//')" --stdout | wl-copy
fi

