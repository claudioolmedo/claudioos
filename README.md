# Claudio OS — One Dollar Computer Emulator

**100% Rust** RV32EC emulator for the [One Dollar Computer](https://github.com/claudioolmedo/claudioos).
Run real board firmware on your Mac/Linux desktop before flashing hardware.

Philosophy and contribution rules: [`ABOUT.md`](ABOUT.md).

## Requirements

- [Rust](https://rustup.rs/) (stable toolchain)
- A desktop environment (the visual emulator opens a native window via `minifb`)

No C toolchain, Python, or shell scripts are required to build or run this repo.

## Quick start

```bash
git clone https://github.com/claudioolmedo/claudioos.git
cd claudioos
cargo test
cargo run --release -- visual-bin testdata/sample.bin
```

`testdata/sample.bin` is an unmodified One Dollar Computer blink firmware image
(the same class of binary you flash to the board).

## Commands

Build the host tool once:

```bash
cargo build --release
```

Then use `./target/release/claudioos <command>` (or `cargo run --release -- <command>`).

| Command | What it does |
| --- | --- |
| `pinout` | Print the One Dollar Computer pin map (`0..19`) |
| `target` | Show the RV32EC board contract (flash/RAM/rules) |
| `blink` | Built-in tiny blink image + GPIO trace |
| `visual-blink` | Window demo of the built-in blink |
| `run-bin <path>` | Load a raw `.bin`, run it, print GPIO events + LED toggles |
| `visual-bin <path>` | **Live** window: run a real `.bin`, LED + boot button |
| `rv32ec-onedollarcomputer model` | Board model contract |
| `rv32ec-onedollarcomputer run <path>` | Same as `run-bin` |
| `rv32ec-onedollarcomputer visual <path>` | Same as `visual-bin` |

Legacy alias: `rv32ec-onedollarboard` still works.

### Trace a binary (no window)

```bash
cargo run --release -- run-bin testdata/sample.bin
```

Useful env vars:

- `CLAUDIOOS_MAX_CYCLES` — cycle budget (default `5000000`)
- `CLAUDIOOS_TRACE_STEPS` — print the first N instruction steps

### Visual emulator (window)

```bash
cargo run --release -- visual-bin testdata/sample.bin
```

Window title: **One Dollar Computer**.

Controls:

- **Click the onboard Button** (silkscreen under ODBPort) — or hold **Space** / **B**
- Momentary switch: pressed only while held
- On real ODC firmware this takes the EXTI → soft-reboot path into **HID bootloader**
- In bootloader mode the active-low LED stays **green / on**
- **Escape** or close the window to quit

## What the emulator models

Enough of the One Dollar Computer contract to run student blink firmware:

- **ISA:** RV32E + compressed (C) + Zicsr (enough for real ODC startup)
- **Memory:** 16 KiB flash / 2 KiB RAM contract (tooling may use larger windows)
- **GPIO** ports A/C/D, including active-low LED on board pin **19** (PD6)
- **SysTick / RCC / AFIO** stubs used by firmware init
- **EXTI + PFIC** path for board pin **13** (PD7) → bootloader soft-reboot
- Pinout overlay + onboard LED visualization

It is a fidelity-focused educational emulator, not a full USB bitbang PHY.

## Layout

```text
Cargo.toml          Host crate (lib + `claudioos` binary)
src/lib.rs          Emulator library (`no_std` core)
src/machine.rs      CPU step / run / interrupts
src/bus.rs          Board bus, GPIO, EXTI/PFIC, peripherals
src/board.rs        One Dollar Computer target contract
src/pinout.rs       Pin 0..19 model
src/bin/claudioos.rs
                    CLI + visual window
testdata/sample.bin Checked-in ODC blink firmware (raw image)
ABOUT.md            Project identity and contribution policy
```

## Tests

```bash
cargo test
```

Includes unit tests plus integration coverage that runs `testdata/sample.bin`
(blink on PD6 and button → bootloader magic at `0x20000400`).

## Using your own firmware

Point `run-bin` / `visual-bin` at any raw RV32EC board image linked like ODC
firmware (vector table at flash base, same GPIO/EXTI conventions):

```bash
cargo run --release -- visual-bin /path/to/your.bin
```

Do not modify the binary for the emulator — load the same bytes you would flash.

## License

MIT (see `Cargo.toml`).
