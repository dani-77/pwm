#!/bin/bash

# pwm passes the active theme's name as $1 (e.g. "gruvbox_dark"). If a
# matching wallpaper exists (d77Lpwm_<theme>.*, or bare d77Lpwm.* for the
# default "d77" theme, which predates the per-theme naming), use it;
# otherwise fall back to picking any wallpaper at random, same as before
# themes had matching art.
theme="$1"
if [ -z "$theme" ] || [ "$theme" = "d77" ]; then
    wallpaper_glob="/etc/xdg/pwm/wallpaper/d77Lpwm.*"
else
    wallpaper_glob="/etc/xdg/pwm/wallpaper/d77Lpwm_$theme.*"
fi
wallpaper=$(compgen -G "$wallpaper_glob" | head -n1)

conky -c ~/.conkyrc &
if [ -n "$wallpaper" ]; then
    feh --bg-fill "$wallpaper" &
else
    feh --bg-fill --randomize /etc/xdg/pwm/wallpaper/ &
fi
synclient TapButton1=1 &
dunst &
udiskie -a &
# No -f (fading): restarting pwm (M-S-r) recreates the status bar with the
# window ID the old one had, and xcompmgr, still fading the old window out,
# then drops the new one and keeps showing a frozen bar.
xcompmgr -c -n &
xautolock -time 5 -locker slock &
# startup.sh runs again on every restart (M-S-r): start redshift only once,
# and -P resets the gamma ramps so its adjustment never stacks.
pgrep -x redshift >/dev/null || redshift -P -l 41.16:-8.62 &
sxhkd -c ~/.config/sxhkd/sxhkdrc
