# Waybar sysinfo plugin
This is a plugin for waybar that displays system information as vertical bars. Inspired by xfce-systemload plugin

This plugin can display following information:
* cpu load, including average load, most loaded core and all cores
* used memory and swap
* network activity
* temperature - max of chosen sensors or all

With reasonable settings

![screenshot](assets/screenshot.webp)

With unreasonable settings

![screenshot, wild mode](assets/sysinfo2.webp)

## Installation
For now only manual installation
```
cargo build --release
cp target/release/libwaybar_sysinfo.so ~/.local/lib/
```

## Config
```jsonc
"modules-right": [
    "cffi/sysinfo",
]
"cffi/sysinfo": {
    "module_path": "/home/USER/.local/lib/libwaybar_sysinfo.so",
    // refresh interval in milliseconds
    "interval_ms": 5000,
    "cpu": {
        // show most loaded core, avg of all cores or all cores
        "show": ["max_core", "avg_core", "all_cores"]
    },
    "mem": {
        "show": ["mem", "swap"]
    },
    "net": {
        // show all networks that match this regexes
        "show": ["eno\\d+", "wlan\\d+"],
        // set a floor for the maximum throughput value
        // ("100%" on the bars, automatically adjusted upwards
        // within a sliding window)
        "floor": 1048576 // in bytes per second, default 5000
    },
    "temp": {
        // show sensor with this name. you can see the list by running `sensors`
        "show": ["Core 1"]
        // show max value for each regex
        "show_max": ["Core .*"]
    },
    // please copy this section as is
    "apps": {
      "signal": [
        {
          "match": "\\([0-9]+\\)$",
          "class": "unread"
        }
      ]
    },
},
```

## Style
This is the default config

```css
.sysinfo-bar {
    padding-left: 5px;
    padding-top: 5px;
    padding-bottom: 5px;
}

/* progress bar */
trough {
    min-height: 3px;
    min-width: 7px;
    border: none;
}

/* colored part of progress bar */
progress {
    border: none;
    min-width: 7px;
}

.cpu progress {
  background-color: #d20f39;
}

.mem progress {
  background-color: #40a02b;
}

.net progress {
  background-color: #1e66f5;
}

.temp progress {
  background-color: #df8e1d;
}
```
