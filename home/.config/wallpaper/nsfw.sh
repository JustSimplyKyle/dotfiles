#!/bin/sh
# wget https://waifu.pics/api/nsfw/waifu -O temp; wget $(cat temp | jq -r ".[]") -O /home/kyle/wallpaper.jpg; rm temp
wget "https://api.waifu.im/random/?is_nsfw=true&selected_tags=waifu&orientation=LANDSCAPE&many=false&full=false" -O temp;  wget $(cat temp | jq -r ".[]" | jq -r ".[]" | jq -r ".url") -O /home/kyle/wallpaper.jpg; rm temp
swaymsg reload
