Trip or Treat
=============

Do you need to run to the subway, or can you have another cookie?

This is the software for a simple display which shows the subway departures for
a particular station in the Stockholm subway. It's written in rust for
Raspberry PI with a framebuffer compatible display (I'm using an AdaFruit TFT).
For development convenience it also has a SDL feature which can be used for
local development.

Local testing
-------------

You need libsdl2 on your system, along with headers. On Ubuntu, you can get
them using:

```
sudo apt-get install libsdl2-dev
```

Cross-compiling for Raspberry PI
--------------------------------

Rust has a nice cross compilation tool called cross which can be installed
using:

```
cargo install cross
```

You can then compile for raspbian as follows:

```
cross build \
    --target armv7-unknown-linux-gnueabihf \
    --no-default-features \
    --features "with-framebuffer"
```
