#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchitectureProfile {
    Rv32ec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesignRule {
    BoardFirst,
    VendorNeutral,
    Educational,
    RustOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectScope {
    pub os_name: &'static str,
    pub board_name: &'static str,
    pub architecture: ArchitectureProfile,
    pub rules: &'static [DesignRule],
}

impl ProjectScope {
    pub const CLAUDIO_OS: Self = Self {
        os_name: "Claudio OS",
        board_name: "One Dollar Board",
        architecture: ArchitectureProfile::Rv32ec,
        rules: &[
            DesignRule::BoardFirst,
            DesignRule::VendorNeutral,
            DesignRule::Educational,
            DesignRule::RustOnly,
        ],
    };

    pub const fn is_vendor_neutral(self) -> bool {
        contains_rule(self.rules, DesignRule::VendorNeutral)
    }

    pub const fn is_rust_only(self) -> bool {
        contains_rule(self.rules, DesignRule::RustOnly)
    }
}

const fn contains_rule(rules: &[DesignRule], expected: DesignRule) -> bool {
    let mut index = 0;
    while index < rules.len() {
        if matches_rule(rules[index], expected) {
            return true;
        }
        index += 1;
    }
    false
}

const fn matches_rule(left: DesignRule, right: DesignRule) -> bool {
    matches!(
        (left, right),
        (DesignRule::BoardFirst, DesignRule::BoardFirst)
            | (DesignRule::VendorNeutral, DesignRule::VendorNeutral)
            | (DesignRule::Educational, DesignRule::Educational)
            | (DesignRule::RustOnly, DesignRule::RustOnly)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_scope_names_os_and_board_separately() {
        let scope = ProjectScope::CLAUDIO_OS;

        assert_eq!(scope.os_name, "Claudio OS");
        assert_eq!(scope.board_name, "One Dollar Board");
        assert_eq!(scope.architecture, ArchitectureProfile::Rv32ec);
        assert!(scope.is_vendor_neutral());
        assert!(scope.is_rust_only());
    }
}
