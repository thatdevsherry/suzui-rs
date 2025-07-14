# suzui-rs

Oxidized version of my [original prototype](https://github.com/thatdevsherry/suzuki_sdl). The car loves this kind of rust.

Made for suzuki baleno (G13BB).

[Ratatui]: https://ratatui.rs

## Motivation and story

After building the initial prototype scanner, I thought of somehow being able to see engine parameters in the car without needing a laptop.

Looked at esp32 etc and embedded, and realized it all went over my head. I was just not in the mood of doing `no_std` dev and I can't solder anything and then add some new screens and what not. So that path was very hard to pursue and remain motivated.

I thought, like any rustacean, "can i build it in rust?" I "could" rewrite the prototype scanner into a rust TUI, but then I'd run it on my laptop again and we're kinda back to square one, just oxidized.

One day I was going over my stereo and realized it has an "AUX" mode which shows blue screen. _HMMM_ what is that. AUX audio should be black or not have a dedicated page. This seems like a video thing. Looked on the internet and saw that the stereo has a "CVBS IN" input for RCA composite video. _HMMMMM_

So I thought why not connect to it. The only thing missing was coding in embedded which I wasn't a fan of, so I took the lazy route and used a raspberry pi to run the code. With pi, I can write for desktop and run it normally using the VAG KKL cable I used for the prototype. Comfy linux environment.

So now my car has a raspberry pi taped onto the ECU, with a VAG KKL cable connection to obd port, power from stereo usb port (undervoltage warnings everywhere), and an ethernet for me to hook up my laptop to ssh into pi for tinkering (fixing) stuff and scp'ing updated binaries to it. Pi's 3.5mm jack connected to 3.5 to RCA cable and the "RED or WHITE" cable is connected to CVBS, because my jack is crap and would not work when fully in, so red/white is the ring closest to having 3.5mm fully inserted to pi so I just rolled with it (don't want a bump to disconnect my very-very-important diagnostics).

Still some stuff to iron out, but we're in a working state now.

Oh did I tell you about how it messes up my immobilizer? Yeah I kinda have to wait for pi to boot up first otherwise it shuts off the car and immo doesn't let engine run again until i cycle key from IGN-ACC-IGN again. Oh also not having pi powered will "NEVER" start the engine, so it's like having an additional key. Custom anti-theft lock and immobilizer combo.

## Showcase

[first run of v0](https://youtu.be/1dXb9Y1NK0k)

[v1](https://youtu.be/kzO5jZieidM?si=MwVlMml7aoIghGfH)

![Image](https://github.com/user-attachments/assets/3a86b3b1-85f6-4aac-82df-3ed14c51612c)


## Setup

ECU <--SDL--> OBD port <--VAG KKL cable--> Pi --RCA composite--> Stereo

## Building/Running

```bash
# Desktop
cargo run -- --simulate

# Pi
cross build --release --target=aarch64-unknown-linux-gnu # binary in target/aarch64-unknown-linux-gnu/release/
```

## License

Copyright (c) Shehriyar Qureshi <thatdevsherry@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
