#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinKind {
    Digital,
    Power,
    Ground,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signal {
    Pc0,
    Pc1,
    Pc2,
    Pc3,
    Pc4,
    Pc5,
    Pc6,
    Pc7,
    Pa1,
    Pa2,
    Pd6,
    Power3v3,
    Ground,
}

impl Signal {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pc0 => "PC0",
            Self::Pc1 => "PC1",
            Self::Pc2 => "PC2",
            Self::Pc3 => "PC3",
            Self::Pc4 => "PC4",
            Self::Pc5 => "PC5",
            Self::Pc6 => "PC6",
            Self::Pc7 => "PC7",
            Self::Pa1 => "PA1",
            Self::Pa2 => "PA2",
            Self::Pd6 => "PD6",
            Self::Power3v3 => "+3V3",
            Self::Ground => "GND",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoardPin {
    pub number: u8,
    pub label: &'static str,
    pub kind: PinKind,
    pub signal: Signal,
    pub package_pad: Option<&'static str>,
    pub rj_pin: Option<&'static str>,
    pub functions: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pinout {
    pub board_name: &'static str,
    pub revision: &'static str,
    pub connector_name: &'static str,
    pub auxiliary_connector_name: &'static str,
    pub pins: &'static [BoardPin],
    pub blink_pin: u8,
}

impl Pinout {
    pub fn pin(self, number: u8) -> Option<&'static BoardPin> {
        self.pins.iter().find(|pin| pin.number == number)
    }

    pub fn blink_pin(self) -> &'static BoardPin {
        self.pin(self.blink_pin)
            .expect("board pinout must contain the blink pin")
    }
}

pub const ONE_DOLLAR_BOARD_PINOUT: Pinout = Pinout {
    board_name: "One Dollar Board",
    revision: "1.004 R1",
    connector_name: "ODB Port",
    auxiliary_connector_name: "RJ45",
    pins: &PINS,
    blink_pin: 20,
};

const PINS: [BoardPin; 13] = [
    BoardPin {
        number: 1,
        label: "1",
        kind: PinKind::Digital,
        signal: Signal::Pc0,
        package_pad: Some("PAD-7"),
        rj_pin: Some("RJ-3"),
        functions: &["GPIO", "T2CH3", "NSS", "T1CH3", "UTX_"],
    },
    BoardPin {
        number: 2,
        label: "2",
        kind: PinKind::Digital,
        signal: Signal::Pc1,
        package_pad: Some("PAD-8"),
        rj_pin: Some("RJ-3"),
        functions: &[
            "GPIO",
            "SDA",
            "NSS",
            "T2CH4_",
            "T2CH1ETR_",
            "T1BKIN_",
            "URX_",
        ],
    },
    BoardPin {
        number: 3,
        label: "3",
        kind: PinKind::Digital,
        signal: Signal::Pc2,
        package_pad: Some("PAD-9"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "SCL", "URTS", "T1ETR", "T2CH2_", "AETR_"],
    },
    BoardPin {
        number: 4,
        label: "4",
        kind: PinKind::Digital,
        signal: Signal::Pc3,
        package_pad: Some("PAD-10"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "T1CH3", "T1CH1N_", "UCTS_"],
    },
    BoardPin {
        number: 5,
        label: "5",
        kind: PinKind::Digital,
        signal: Signal::Pc4,
        package_pad: Some("PAD-11"),
        rj_pin: Some("RJ-6"),
        functions: &["GPIO", "A2", "T1CH4", "MCO", "T1CH1CH2N_"],
    },
    BoardPin {
        number: 6,
        label: "6",
        kind: PinKind::Digital,
        signal: Signal::Pc5,
        package_pad: Some("PAD-12"),
        rj_pin: Some("RJ-5"),
        functions: &["GPIO", "SCK", "T1ETR", "T2CH1ETR_", "SCL_", "T1CH3_"],
    },
    BoardPin {
        number: 7,
        label: "7",
        kind: PinKind::Digital,
        signal: Signal::Pc6,
        package_pad: Some("PAD-13"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "MOSI", "T1CH1CH3N_", "UCTS_", "SDA_"],
    },
    BoardPin {
        number: 8,
        label: "8",
        kind: PinKind::Digital,
        signal: Signal::Pc7,
        package_pad: Some("PAD-14"),
        rj_pin: Some("RJ-1"),
        functions: &["GPIO", "MISO", "T1CH2_", "T2CH2_", "URTS_"],
    },
    BoardPin {
        number: 9,
        label: "9",
        kind: PinKind::Digital,
        signal: Signal::Pa1,
        package_pad: Some("PAD-2"),
        rj_pin: None,
        functions: &["GPIO", "A1", "OSCI", "T1CH2", "OPN0"],
    },
    BoardPin {
        number: 10,
        label: "10",
        kind: PinKind::Digital,
        signal: Signal::Pa2,
        package_pad: Some("PAD-3"),
        rj_pin: None,
        functions: &["GPIO", "A0", "OSCO", "T1CH2N", "OPP0", "AETR2_"],
    },
    BoardPin {
        number: 11,
        label: "11",
        kind: PinKind::Power,
        signal: Signal::Power3v3,
        package_pad: None,
        rj_pin: None,
        functions: &["POWER"],
    },
    BoardPin {
        number: 12,
        label: "12",
        kind: PinKind::Ground,
        signal: Signal::Ground,
        package_pad: None,
        rj_pin: None,
        functions: &["GROUND"],
    },
    BoardPin {
        number: 20,
        label: "20",
        kind: PinKind::Digital,
        signal: Signal::Pd6,
        package_pad: Some("PAD-20"),
        rj_pin: None,
        functions: &["GPIO", "A6", "URX", "T2CH3", "UTX_"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_blink_pin_from_board_pinout() {
        let blink = ONE_DOLLAR_BOARD_PINOUT.blink_pin();

        assert_eq!(blink.number, 20);
        assert_eq!(blink.signal, Signal::Pd6);
        assert!(blink.functions.contains(&"GPIO"));
    }
}
