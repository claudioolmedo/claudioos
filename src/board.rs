#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstructionSet {
    Rv32ec,
}

impl InstructionSet {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rv32ec => "RV32EC",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityRule {
    BoardContractFirst,
    StableRustPrograms,
    ReplaceableMicrocontroller,
    PinoutCompatibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoardTarget {
    pub name: &'static str,
    pub cli_name: &'static str,
    pub model: BoardModel,
    pub instruction_set: InstructionSet,
    pub flash_bytes: usize,
    pub ram_bytes: usize,
    pub rules: &'static [CompatibilityRule],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoardModel {
    pub name: &'static str,
    pub revision: &'static str,
    pub compatibility_family: &'static str,
}

pub const ONE_DOLLAR_BOARD_MODEL_1_004: BoardModel = BoardModel {
    name: "One Dollar Computer",
    revision: "1.004 R1",
    compatibility_family: "One Dollar Computer 1.x",
};

impl BoardTarget {
    pub const RV32EC_ONE_DOLLAR_BOARD: Self = Self {
        name: "One Dollar Computer",
        cli_name: "rv32ec-onedollarcomputer",
        model: ONE_DOLLAR_BOARD_MODEL_1_004,
        instruction_set: InstructionSet::Rv32ec,
        // One Dollar Computer 1.004 contract (CH32V003-class): 16 KiB flash, 2 KiB RAM.
        // BoardBus generics may allocate larger windows for tooling; this is the
        // portable contract student programs should assume.
        flash_bytes: 16_384,
        ram_bytes: 2_048,
        rules: &[
            CompatibilityRule::BoardContractFirst,
            CompatibilityRule::StableRustPrograms,
            CompatibilityRule::ReplaceableMicrocontroller,
            CompatibilityRule::PinoutCompatibility,
        ],
    };

    pub const fn keeps_programs_portable(self) -> bool {
        contains_rule(self.rules, CompatibilityRule::StableRustPrograms)
            && contains_rule(self.rules, CompatibilityRule::ReplaceableMicrocontroller)
    }
}

const fn contains_rule(rules: &[CompatibilityRule], expected: CompatibilityRule) -> bool {
    let mut index = 0;
    while index < rules.len() {
        if matches_rule(rules[index], expected) {
            return true;
        }
        index += 1;
    }
    false
}

const fn matches_rule(left: CompatibilityRule, right: CompatibilityRule) -> bool {
    matches!(
        (left, right),
        (
            CompatibilityRule::BoardContractFirst,
            CompatibilityRule::BoardContractFirst
        ) | (
            CompatibilityRule::StableRustPrograms,
            CompatibilityRule::StableRustPrograms
        ) | (
            CompatibilityRule::ReplaceableMicrocontroller,
            CompatibilityRule::ReplaceableMicrocontroller
        ) | (
            CompatibilityRule::PinoutCompatibility,
            CompatibilityRule::PinoutCompatibility
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_board_target_without_vendor_identity() {
        let target = BoardTarget::RV32EC_ONE_DOLLAR_BOARD;

        assert_eq!(target.name, "One Dollar Computer");
        assert_eq!(target.cli_name, "rv32ec-onedollarcomputer");
        assert_eq!(target.model.revision, "1.004 R1");
        assert_eq!(target.instruction_set.name(), "RV32EC");
        assert!(target.keeps_programs_portable());
    }
}
