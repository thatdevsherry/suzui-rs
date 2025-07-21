# suzui-rs

Oxidized version of my [original prototype](https://github.com/thatdevsherry/suzuki_sdl).

Made for suzuki baleno (G13BB).

Built with rust and [ratatui](https://ratatui.rs/).

## Showcase

![Image](https://github.com/user-attachments/assets/3a86b3b1-85f6-4aac-82df-3ed14c51612c)

[first run of v0](https://youtu.be/1dXb9Y1NK0k)

[v1](https://youtu.be/kzO5jZieidM?si=MwVlMml7aoIghGfH)

## Features

| Parameter                  | Description                                                      | Unit    |
| -------------------------- | -----------------------------------------------------------------| ------- |
| Engine speed               | How much engine is vrooming                                      | RPM     |
| Desired idle               | Intended idle by ECU                                             | RPM     |
| ISC flow duty              | How much IACV is open                                            | %       |
| Inj. pulse width           | pulse width of injector in cylinder 1                            | ms      |
| Ignition advance           | Ignition advance commanded by ECU                                | BTDC    |
| IAT                        | Intake air temperature                                           | C       |
| ECT                        | Engine coolant temperature                                       | C       |
| Absolute throttle position | Throttle position based on full TPS range                        | %       |
| Throttle angle             | Calculated throttle (butterfly valve) angle                      | degrees |
| MAP                        | Manifold absolute pressure                                       | kPa     |
| BARO                       | Barometric pressure, taken from MAP before first crank           | kPa     |
| Calc. load                 | Calculated engine load (accurate approx. based on avail. data)   | %       |
| Battery voltage            | Battery voltage read by ECU                                      | V       |
| Vehicle speed (VSS)        | How fast car actually go                                         | km/h    |
| EL                         | Electric load                                                    | ON/OFF  |
| AC                         | AC switch                                                        | ON/OFF  |
| PSP                        | Power steering pump switch                                       | ON/OFF  |
| RAD                        | Radiator fan                                                     | ON/OFF  |
| Fuel cut                   | Deceleration fuel cut off (DFCO). Calc. from inj. pw             | ON/OFF  |
| Total fuel used<sup>*</sup>            | Track fuel use when engine running until explicit reset          | litres  |
| Cumulative fuel<sup>*</sup>            | Fuel used when engine running and vehicle moving                 | litres  |
| Cumulative distance<sup>*</sup>        | Distance covered by car (odometer) until explicit reset          | ON/OFF  |
| Instant fuel consumption<sup>*</sup>   | Instantaneous fuel consumption at specific moment in time        | L/100km |
| Long-term fuel consumption<sup>*</sup> | Long-term fuel consumption using only for moving car             | L/100km |

Parameters marked <sup>*<sup> are not stable

<details>
  <summary>Not implemented</summary>

  ### TPS Voltage

  Formula assumes fixed voltage (5V). Although it's almost 5V, any deviation from it results in inaccurate reading. TPS does not give back input voltage so this formula/parameter is useless.
  It is way better to use "Absolute throttle position" parameter as that provides output of 0-100% of the input voltage, whatever it may be.
</details>

## Motivation and story

After building the initial prototype scanner, I thought of somehow being able to see engine parameters in the car without needing a laptop.

Looked at esp32 etc and embedded, and realized it all went over my head. I was just not in the mood of doing `no_std` dev and I can't solder anything and then add some new screens and what not. So that path was very hard to pursue and remain motivated.

I thought, like any rustacean, "can i build it in rust?" I "could" rewrite the prototype scanner into a rust TUI, but then I'd run it on my laptop again and we're kinda back to square one, just oxidized.

One day I was going over my stereo and realized it has an "AUX" mode which shows blue screen. _HMMM_ what is that. AUX audio should be black or not have a dedicated page. This seems like a video thing. Looked on the internet and saw that the stereo has a "CVBS IN" input for RCA composite video. _HMMMMM_

So I thought why not connect to it. The only thing missing was coding in embedded which I wasn't a fan of, so I took the lazy route and used a raspberry pi to run the code. With pi, I can write for desktop and run it normally using the VAG KKL cable I used for the prototype. Comfy linux environment.

So now my car has a raspberry pi taped onto the ECU, with a VAG KKL cable connection to obd port, power from stereo usb port (undervoltage warnings everywhere), and an ethernet for me to hook up my laptop to ssh into pi for tinkering (fixing) stuff and scp'ing updated binaries to it. Pi's 3.5mm jack connected to 3.5 to RCA cable and the "RED or WHITE" cable is connected to CVBS, because my jack is crap and would not work when fully in, so red/white is the ring closest to having 3.5mm fully inserted to pi so I just rolled with it (don't want a bump to disconnect my very-very-important diagnostics).

Oh did I tell you about how it messes up my immobilizer? Yeah I kinda have to wait for pi to boot up first otherwise it shuts off the car and immo doesn't let engine run again until i cycle key from IGN-ACC-IGN again. Oh also not having pi powered will "NEVER" start the engine, so it's like having an additional key. Custom anti-theft lock and immobilizer combo.

## Setup

<img width="2501" height="1522" alt="image" src="https://github.com/user-attachments/assets/8a0214b6-6774-440a-b273-18c9281db501" />

<details>
  <summary>Future improvements</summary>

  ### DTCs
  
  Forgot we can use popups. Easiest way is to just show a centered popup with all DTCs found. Either do it one time at boot or on each poll (former might be nice since DTCs aren't popping up all the time).
  
  But need to handle **current** vs **history** codes.

  I'm thinking for **current**, we can instead show the fault code in the relevant section (highlighting red background or full red gauge if its a gauge.

  - ECT (high/low): Show "ECT high" or "ECT low" in the ECT row inside "TEMPERATURES" block
  - IAT (high/low): Same as above but instead in IAT row
  - TPS (high/low): Same as above but instead in throttle gauge inside "THROTTLE" block
  - VSS fault: Show in vehicle speed row
  - MAP (high/low): Show in "LOAD" block
  - IAC fault: Show in ISC flow duty gauge
  - Ignition fault: Show in "IGN ADV" row
  - Crankshaft fault: Show in RPM gauge
  - Camshaft fault: Show in RPM gauge label (rpm gauge will still work if CKP sensor is fine)
  - Injector fault: Show in inj. pw gauge

  The above feels too complex though. Have to think.

  Current codes should be visible at all times since they're important.

  For history codes, we can setup a clear codes logic on bootup that:

  - on boot, check for history codes, if present, show a popup showing the codes and a timer (30s) before they're cleared.

  This is just so I can see history code if any got popped up randomly, but then have it get cleared since it isn't valid anymore. Useful for intermittent issues that cause a code but then go away. Will let me know that a code was triggered and I can investigate.

  Fun thing that could be done: instead of adding buttons into dashboard and connect to pi, what if i setup listener to say, turn lights on and off quickly, say, 5 times so EL bool changes quickly, and then set a task that if EL changes 5 times in 5 seconds, run clear codes command or something XD such a stupid thing but i love the idea

  ### Fuel economy (Instant or long-term)

  If instant could be calculated, need to know injector fuel flow rate which I don't. Think I can then calculate using "inj. pw, RPM, VSS, fuel flow rate".

  For long-term, would require writing to card. But instant fuel economy calculation is a pre-requisite.

  ### Gear indicator

  Use speed and rpm to show identified gear, when clutch pressed it'll just go down,but otherwise just a fun thing to add maybe
</details>

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
