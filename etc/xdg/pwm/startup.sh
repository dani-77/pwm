#!/bin/bash

conky -c ~/.conkyrc &
feh --bg-fill --randomize /etc/xdg/wallpaper/ &
synclient TapButton1=1 &
dunst &
udiskie -a &
xcompmgr -c -f -n &
xautolock -time 5 -locker slock &
redshift -l 41.16:-8.62 &
sxhkd -c ~/.config/sxhkd/sxhkdrc
