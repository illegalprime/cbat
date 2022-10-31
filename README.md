# The C-Bat

Play Audio Clips via I2S w/ an Arduino ItsyBitsy M0 in Rust

The HAL for rust was really nice to use, although it didn't have I2S support, so I had to implement that myself.

## To Use

[Nix](https://nixos.org/download.html) is wonderful and you can run this with a single command if you have it installed (on Linux):

```
nix-shell shell.nix --run 'cargo hf2 --release'
```

## In Action

https://www.youtube.com/shorts/0nkotU7HYhI
