# Conky

Just in case you want to display the keybinds on your screen, copy conkyrc to ~/ as follows:

```
$ cp conkyrc ~/.conkyrc
```

Then add this line to the startup script:

```
# sed -i '3i conky -c ~/.conkyrc &' /etc/xdg/wmd77/startup.sh
```
    
so that the conky knows what to read and start on startup.

Lastly, install conky to have it displaying the keybinds.