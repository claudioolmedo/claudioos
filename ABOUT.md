# Claudio OS

Claudio OS is an experimental operating system project for the One Dollar Board, built from scratch as an educational RISC-V platform.

**CLAUDIO OS** means **Collaborative Learning Agentic Universal Device Interface Operating System**.

The project exists to make low-level computing easier to study, modify, emulate, and understand. The long-term goal is to create a small, clear, open system where the same ideas can move from source code, to emulator, to physical board, while remaining simple enough for students, makers, and contributors to follow.

Agentic means contributors may use their own agents to help understand the project, write Rust code, run checks, and prepare contributions. The repository itself should remain focused on Claudio OS and its emulator.

## Core Rule

**Claudio OS must remain 100% Rust.**

No C. No C++. No JavaScript. No Python. No shell scripts. No generated vendor SDKs. No helper tools in other languages.

Contributions written in any language other than Rust will not be accepted.

This rule applies to the operating system, emulator, tools, examples, build logic, project code, and tests. The goal is to build a complete Rust-first educational stack, not a Rust wrapper around legacy code.

The only accepted Markdown exception is this project overview file.

Rust is part of the project identity because it helps the One Dollar Board ecosystem stay portable, maintainable, and less dependent on any single chip vendor or board revision.

## Project Goals

Claudio OS is being designed around these goals:

- create a tiny educational operating system from first principles;
- support the One Dollar Board as the main development target;
- provide an emulator that can run board programs before flashing hardware;
- make embedded programming easier to visualize and debug;
- keep the architecture portable across compatible RISC-V microcontrollers;
- help learners understand what happens below ordinary application code;
- build a public foundation for future boards without breaking older projects.

## Why This Exists

Small boards are powerful teaching tools, but their ecosystems often become tied to one specific vendor, SDK, toolchain, or set of examples. Claudio OS is intended to reduce that dependency.

The One Dollar Board should be able to evolve over time while keeping a stable educational and software identity. A Rust-based operating system and emulator can help preserve compatibility, document behavior, and make future hardware revisions easier to support.

The first generation may be limited. Future generations may have more memory, peripherals, or capabilities. Claudio OS should help connect those generations through a shared programming model and a careful compatibility strategy.

## RV32EC One Dollar Board Emulator

The emulator target is **RV32EC One Dollar Board**.

The first concrete board model is **One Dollar Board 1.004 R1**.

This name is intentional. The project should target the One Dollar Board contract, not a specific chip identity. A physical board revision may change its microcontroller later, but Rust programs should keep working when the board still provides the same instruction profile, pinout contract, memory expectations, and board behavior.

The compatibility layer should therefore be designed around:

- the RV32EC instruction profile;
- the One Dollar Board 1.004 R1 board-level pinout and observable behavior;
- stable Rust examples;
- explicit memory and peripheral contracts;
- emulation before hardware flashing;
- future board revisions that preserve older projects where practical.

The emulator is not just a debugging utility. It is the compatibility reference for the board ecosystem.

For 1.004 R1, the public connector contract uses board pins `0..19`: `0..9` are PC0-PC7/PA1/PA2, `10` is +3V3, `11` is GND, `12..15` are PD1/PD7/PD0/PD2, `16..18` are NC, and `19` is PD6 / blink LED. Student-facing code should depend on this board contract first and map to chip pins through the selected board revision.

## Emulator First

The emulator is a central part of the project.

Before code runs on physical hardware, it should be possible to run it locally, inspect what it does, and visualize the result. This makes the board easier to teach, easier to debug, and easier to use in environments where hardware is not immediately available.

Early emulator goals:

- execute simple RV32EC programs;
- model registers, memory, and basic board behavior;
- visualize GPIO activity;
- run a blink example;
- load real board binaries;
- make failures visible instead of silent.

## Operating System Direction

The operating system should start very small.

Initial focus:

- boot flow;
- memory layout;
- minimal task execution;
- GPIO control;
- deterministic examples;
- clear internal structure;
- strong documentation inside the Rust code.

The project should grow carefully. New abstractions should only be added when they make the system easier to understand, test, or extend.

## One Dollar Board Compatibility

The One Dollar Board is the hardware identity of the project.

Claudio OS should support the idea that board generations can improve over time while keeping older projects meaningful. Compatibility matters because people may build projects, lessons, tools, and examples around early versions of the board.

The project should therefore avoid unnecessary dependence on a single chip name, vendor-specific identity, or fragile assumptions that would make future revisions harder.

The rule is board compatibility first: Rust code should target the One Dollar Board interface. The physical microcontroller is an implementation detail when the board contract remains compatible.

## Repository Principles

- Rust only.
- Keep the code understandable.
- Prefer small, testable modules.
- Avoid hidden magic in build steps.
- Do not add C or C++ support paths.
- Do not add source files in other programming languages.
- Do not add generated vendor SDK code.
- Do not submit local notes, scratch plans, or generated Markdown files.
- Do not make hardware assumptions without documenting them.
- Favor portability across compatible RISC-V targets.
- Keep examples simple enough to teach.

## Contribution Policy

Contributions are welcome when they support the educational and Rust-first direction of the project.

Accepted contribution areas include:

- Rust emulator improvements;
- Rust operating system code;
- Rust tooling for board visualization;
- Rust examples;
- tests;
- documentation;
- compatibility notes;
- issue reports and design discussion.

Not accepted:

- C firmware;
- C++ firmware;
- JavaScript tools;
- Python tools;
- shell-script build paths;
- vendor SDK dumps;
- build systems that require C or C++;
- examples that depend on non-Rust source code;
- local memory files;
- generated Markdown planning files;
- changes that tie the project identity to one specific chip vendor.

## Current Status

Claudio OS is early and experimental.

The project currently focuses on building the foundation: instruction decoding, machine state, board modeling, pinout visualization, real binary loading, and a visual blink demonstration.

## Long-Term Vision

The long-term vision is a complete educational loop:

1. write Rust code;
2. compile for the One Dollar Board;
3. run it in the emulator;
4. inspect board behavior visually;
5. flash physical hardware;
6. keep the same project compatible as the board evolves.

Claudio OS is an experiment in making extremely low-cost computing more open, more teachable, and more durable.
