## [2.0.1] - 2025-07-22

### 🐛 Bug Fixes

- *(README)* Fix funny formatting
- *(README)* Fuel related param desc.
- Show fuel consumption on engine off

### ⚙️ Miscellaneous Tasks

- *(README)* Add info about unstable params
- *(README)* Remove fuel stuff from TODOs
- *(README)* Remove gear indicator idea
## [2.0.0] - 2025-07-21

### 🚀 Features

- Calculate instant fuel consumption
- Flags section lookin real nice
- Fuel and distance calculation
- Show fuel used in fuel/ign block
- Show odo and fuel consumption
- [**breaking**] Persisted distance and fuel calculation
- [**breaking**] Add trip meter reset logic
- Trip calculation improvements
- Show fuel flow rate w/ fuel used

### 🐛 Bug Fixes

- Colors for rpm and batt
- Change distance_fuel file path
- Separate partition path
- Have run script show error for 5 second
- Inj. flow rate and calc.
- Update path to use home dir

### ⚙️ Miscellaneous Tasks

- *(README)* Add feature section
- *(README)* Add ratatui link
- *(README)* Add fuel cut feature
- *(README)* Add future improv. section
- Clippy happy
- TtyUSB0 as string
- Fmt
- Bump to 2.0.0
## [1.0.0] - 2025-07-14

### 🚀 Features

- Adj. for big font, low res stereo display
- No padding for FAN ON display
- Script to autorestart app
- [**breaking**] Set event poll to 0
- Inj pw gauge limit to 20ms
- Rust 2024 stuff
- [**breaking**] Set event poll to 0 if not simulating
- *(engine)* Round engine rpm to 50s
- Cosmetic changes
- *(Cargo.toml)* Release build optimization
- UI changes for bigger font on smol screen

### 🐛 Bug Fixes

- *(sdl)* Do not query fault codes in live data
- *(ci)* Vibe update CI

### ⚙️ Miscellaneous Tasks

- Make clippy happy
- *(README)* Add diagram and instructions
- *(README)* Add nice diagram and stuff
- Cargo fmt
- *(Cargo.toml)* Bump to v1.0.0
