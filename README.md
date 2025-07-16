# suzui-rs

Oxidized version of my [original prototype](https://github.com/thatdevsherry/suzuki_sdl).

Made for suzuki baleno (G13BB).

Built with rust and [ratatui](https://ratatui.rs/).

## Showcase

![Image](https://github.com/user-attachments/assets/3a86b3b1-85f6-4aac-82df-3ed14c51612c)

[first run of v0](https://youtu.be/1dXb9Y1NK0k)

[v1](https://youtu.be/kzO5jZieidM?si=MwVlMml7aoIghGfH)

## Features

| Parameter                  | Description                                            | Unit    |
| -------------------------- | ------------------------------------------------------ | ------- |
| Engine speed               | How much engine is vrooming                            | RPM     |
| Desired idle               | Intended idle by ECU                                   | RPM     |
| ISC flow duty              | How much IACV is open                                  | %       |
| Inj. pulse width           | pulse width of injector in cylinder 1                  | ms      |
| Ignition advance           | Ignition advance commanded by ECU                      | BTDC    |
| IAT                        | Intake air temperature                                 | C       |
| ECT                        | Engine coolant temperature                             | C       |
| Absolute throttle position | Throttle position based on full TPS range              | %       |
| Throttle angle             | Calculated throttle (butterfly valve) angle            | degrees |
| MAP                        | Manifold absolute pressure                             | kPa     |
| BARO                       | Barometric pressure, taken from MAP before first crank | kPa     |
| Calc. load                 | Calculated engine load (accurate approx. based on avail. data)                 | %       |
| Battery voltage            | Battery voltage read by ECU                            | V       |
| Vehicle speed (VSS)        | How fast car actually go                               | km/h    |
| EL                         | Electric load                                          | ON/OFF  |
| AC                         | AC switch                                              | ON/OFF  |
| PSP                        | PSP switch                                             | ON/OFF  |
| RAD                        | Radiator fan                                           | ON/OFF  |

<details>
  <summary>Not implemented</summary>

  ### TPS Voltage

  Formula assumes fixed voltage (5V). Although it's almost 5V, any deviation from it results in inaccurate reading. TPS does not give back input voltage so this formula/parameter is useless.
  It is way better to use "Absolute throttle position" parameter as that provides output of 0-100% of the input voltage, whatever it may be.

  ### DTCs
  I can't connect a big keyboard in car, would have to setup some buttons to switch b/w TUI tabs (not implemented). However I don't have it planned since most (but not all) of the DTCs can be identified in live data page.

  The following are identifiable:

  - ECT/IAT (high/low): gauge will show min or max (-40,119)
  - TPS (high/low): gauge will show min or max (0,100)
  - VSS fault: 0 speed when moving (also shown on speedo i guess)
  - MAP (high/low): MAP value will be min or max (-20, 146.63)
  - IAC fault: ISC flow duty will be min (1%)

  The following cannot be identified as of now:

  - Ignition fault: Ignition advance is what's commanded by ECU so it might not go to 0 to indicate it. Have not replicated this DTC in my car
  - Crankshaft fault: Car will not start. It might be identifiable as when cranking the engine speed will remain 0. Have not replicated to confirm since car might use CMP to calculate RPM (pretty sure it can't w/o this). I can't identify it since pi restarts when car is cranking so I wouldn't be able to see the RPM value 😆
  - Camshaft fault: Car might still start (in one of my old tests) so this would not be identifiable with any other parameter.
  - Injector fault: Pretty sure inj. pw parameter will still be working, so this is not identifiable. I haven't replicated this DTC in my car to confirm.

  So for DTCs, it's just better to have a small wire in car to short diagnostic pins in fuse box, and see the check engine light blink code if the DTC is not in the "identifiable" list 🫡

  Although I've also worked on an android app, which shows DTCs. So for me I'd just yank out vag cable from pi and connect it to my phone (if i carry OTG) and see DTCs from my phone.
</details>

## Motivation and story

After building the initial prototype scanner, I thought of somehow being able to see engine parameters in the car without needing a laptop.

Looked at esp32 etc and embedded, and realized it all went over my head. I was just not in the mood of doing `no_std` dev and I can't solder anything and then add some new screens and what not. So that path was very hard to pursue and remain motivated.

I thought, like any rustacean, "can i build it in rust?" I "could" rewrite the prototype scanner into a rust TUI, but then I'd run it on my laptop again and we're kinda back to square one, just oxidized.

One day I was going over my stereo and realized it has an "AUX" mode which shows blue screen. _HMMM_ what is that. AUX audio should be black or not have a dedicated page. This seems like a video thing. Looked on the internet and saw that the stereo has a "CVBS IN" input for RCA composite video. _HMMMMM_

So I thought why not connect to it. The only thing missing was coding in embedded which I wasn't a fan of, so I took the lazy route and used a raspberry pi to run the code. With pi, I can write for desktop and run it normally using the VAG KKL cable I used for the prototype. Comfy linux environment.

So now my car has a raspberry pi taped onto the ECU, with a VAG KKL cable connection to obd port, power from stereo usb port (undervoltage warnings everywhere), and an ethernet for me to hook up my laptop to ssh into pi for tinkering (fixing) stuff and scp'ing updated binaries to it. Pi's 3.5mm jack connected to 3.5 to RCA cable and the "RED or WHITE" cable is connected to CVBS, because my jack is crap and would not work when fully in, so red/white is the ring closest to having 3.5mm fully inserted to pi so I just rolled with it (don't want a bump to disconnect my very-very-important diagnostics).

Still some stuff to iron out, but we're in a working state now.

Oh did I tell you about how it messes up my immobilizer? Yeah I kinda have to wait for pi to boot up first otherwise it shuts off the car and immo doesn't let engine run again until i cycle key from IGN-ACC-IGN again. Oh also not having pi powered will "NEVER" start the engine, so it's like having an additional key. Custom anti-theft lock and immobilizer combo.

## Setup

<img width="2501" height="1522" alt="image" src="https://github.com/user-attachments/assets/8a0214b6-6774-440a-b273-18c9281db501" />

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
