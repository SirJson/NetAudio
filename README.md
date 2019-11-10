# NetAudio

## What is this?

A small application that redirects all of your windows audio output via udp to an other device like the Raspberry Pi. I created this in order to stream everything to my Pi DAC for better sound quality. Kinda like as a network attached soundcard.

There is build for LAN use over Ethernet, so no the network implementation is very much just fire and forget. I think this wouldn't work over wireless or the internet at the moment.

## Binaries

Pre-build binaries for Raspbian Buster and Windows can be found (https://github.com/SirJson/NetAudio/releases)[here]

## Build

Building it should be straightforward. Just run cargo build both folder. The `netaudio-server` is the application that will playback audio it receives and `netaudio-source` is the "client" that will generate audio.

The `netaudio-source` application also needs to know which server it should connect. This can be controlled via `--target`

### netaudio-server commandline options

```
Options:
    -i, --ip IP         ip the server will bind to
    -p, --port PORT     port the server will bind to
    -h, --help          print this help menu
```

### netaudio-source commandline options

```
Options:
    -t, --target IP     the target audio server
    -p, --port PORT     the target port
    -h, --help          print this help menu
```