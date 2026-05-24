#![no_std]

pub mod board;
pub mod bus;
pub mod instruction;
pub mod machine;
pub mod pinout;
pub mod project;
pub mod register;

pub use board::{
    BoardModel, BoardTarget, CompatibilityRule, InstructionSet, ONE_DOLLAR_BOARD_MODEL_1_004,
};
pub use bus::{BoardBus, Bus, BusFault, GpioPort, RamBus, GPIO_OUT};
pub use instruction::{Instruction, Opcode};
pub use machine::{Event, EventKind, Machine, RunLimit, StopReason};
pub use pinout::{BoardPin, PinKind, Pinout, Signal, ONE_DOLLAR_BOARD_PINOUT};
pub use project::{ArchitectureProfile, DesignRule, ProjectScope};
pub use register::{Register, RegisterError, REGISTER_COUNT};
