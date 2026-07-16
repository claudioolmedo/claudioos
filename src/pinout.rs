#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinKind {
    Digital,
    Power,
    Ground,
    NotConnected,
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
    Pd0,
    Pd1,
    Pd2,
    Pd6,
    Pd7,
    Power3v3,
    Ground,
    NotConnected,
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
            Self::Pd0 => "PD0",
            Self::Pd1 => "PD1",
            Self::Pd2 => "PD2",
            Self::Pd6 => "PD6",
            Self::Pd7 => "PD7",
            Self::Power3v3 => "+3V3",
            Self::Ground => "GND",
            Self::NotConnected => "NC",
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
    board_name: "One Dollar Computer",
    revision: "1.004 R1",
    connector_name: "ODB Port",
    auxiliary_connector_name: "RJ45",
    pins: &PINS,
    blink_pin: 19,
};

const PINS: [BoardPin; 20] = [
    BoardPin {
        number: 0,
        label: "0",
        kind: PinKind::Digital,
        signal: Signal::Pc0,
        package_pad: Some("MCU-7"),
        rj_pin: Some("RJ-3"),
        functions: &["GPIO", "T2CH3", "NSS", "T1CH3", "UTX_"],
    },
    BoardPin {
        number: 1,
        label: "1",
        kind: PinKind::Digital,
        signal: Signal::Pc1,
        package_pad: Some("MCU-8"),
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
        number: 2,
        label: "2",
        kind: PinKind::Digital,
        signal: Signal::Pc2,
        package_pad: Some("MCU-9"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "SCL", "URTS", "T1ETR", "T2CH2_", "AETR_"],
    },
    BoardPin {
        number: 3,
        label: "3",
        kind: PinKind::Digital,
        signal: Signal::Pc3,
        package_pad: Some("MCU-10"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "T1CH3", "T1CH1N_", "UCTS_"],
    },
    BoardPin {
        number: 4,
        label: "4",
        kind: PinKind::Digital,
        signal: Signal::Pc4,
        package_pad: Some("MCU-11"),
        rj_pin: Some("RJ-6"),
        functions: &["GPIO", "A2", "T1CH4", "MCO", "T1CH1CH2N_"],
    },
    BoardPin {
        number: 5,
        label: "5",
        kind: PinKind::Digital,
        signal: Signal::Pc5,
        package_pad: Some("MCU-12"),
        rj_pin: Some("RJ-5"),
        functions: &["GPIO", "SCK", "T1ETR", "T2CH1ETR_", "SCL_", "T1CH3_"],
    },
    BoardPin {
        number: 6,
        label: "6",
        kind: PinKind::Digital,
        signal: Signal::Pc6,
        package_pad: Some("MCU-13"),
        rj_pin: Some("RJ-2"),
        functions: &["GPIO", "MOSI", "T1CH1CH3N_", "UCTS_", "SDA_"],
    },
    BoardPin {
        number: 7,
        label: "7",
        kind: PinKind::Digital,
        signal: Signal::Pc7,
        package_pad: Some("MCU-14"),
        rj_pin: Some("RJ-1"),
        functions: &["GPIO", "MISO", "T1CH2_", "T2CH2_", "URTS_"],
    },
    BoardPin {
        number: 8,
        label: "8",
        kind: PinKind::Digital,
        signal: Signal::Pa1,
        package_pad: Some("MCU-2"),
        rj_pin: None,
        functions: &["GPIO", "A1", "OSCI", "T1CH2", "OPN0"],
    },
    BoardPin {
        number: 9,
        label: "9",
        kind: PinKind::Digital,
        signal: Signal::Pa2,
        package_pad: Some("MCU-3"),
        rj_pin: None,
        functions: &["GPIO", "A0", "OSCO", "T1CH2N", "OPP0", "AETR2_"],
    },
    BoardPin {
        number: 10,
        label: "10",
        kind: PinKind::Power,
        signal: Signal::Power3v3,
        package_pad: None,
        rj_pin: Some("RJ-8"),
        functions: &["POWER"],
    },
    BoardPin {
        number: 11,
        label: "11",
        kind: PinKind::Ground,
        signal: Signal::Ground,
        package_pad: Some("MCU-0"),
        rj_pin: Some("RJ-7"),
        functions: &["GROUND"],
    },
    BoardPin {
        number: 12,
        label: "12",
        kind: PinKind::Digital,
        signal: Signal::Pd1,
        package_pad: Some("MCU-15"),
        rj_pin: None,
        functions: &["GPIO", "SWIO", "AETR2", "T1CH3N", "SCL_", "URX_"],
    },
    BoardPin {
        number: 13,
        label: "13",
        kind: PinKind::Digital,
        signal: Signal::Pd7,
        package_pad: Some("MCU-1"),
        rj_pin: None,
        functions: &["GPIO", "NRST", "T2CH4", "OPP1", "UCK"],
    },
    BoardPin {
        number: 14,
        label: "14",
        kind: PinKind::Digital,
        signal: Signal::Pd0,
        package_pad: Some("MCU-5"),
        rj_pin: None,
        functions: &["GPIO", "T1CH1N", "OPN1", "SDA_", "UTX_"],
    },
    BoardPin {
        number: 15,
        label: "15",
        kind: PinKind::Digital,
        signal: Signal::Pd2,
        package_pad: Some("MCU-16"),
        rj_pin: None,
        functions: &["GPIO", "A3", "T1CH1", "T2CH3", "T1CH2N"],
    },
    BoardPin {
        number: 16,
        label: "16",
        kind: PinKind::NotConnected,
        signal: Signal::NotConnected,
        package_pad: None,
        rj_pin: None,
        functions: &["NC"],
    },
    BoardPin {
        number: 17,
        label: "17",
        kind: PinKind::NotConnected,
        signal: Signal::NotConnected,
        package_pad: None,
        rj_pin: None,
        functions: &["NC"],
    },
    BoardPin {
        number: 18,
        label: "18",
        kind: PinKind::NotConnected,
        signal: Signal::NotConnected,
        package_pad: None,
        rj_pin: None,
        functions: &["NC"],
    },
    BoardPin {
        number: 19,
        label: "19",
        kind: PinKind::Digital,
        signal: Signal::Pd6,
        package_pad: Some("MCU-20"),
        rj_pin: None,
        functions: &["GPIO", "A6", "URX", "T2CH4_", "UTX_"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_blink_pin_from_board_pinout() {
        let blink = ONE_DOLLAR_BOARD_PINOUT.blink_pin();

        assert_eq!(blink.number, 19);
        assert_eq!(blink.signal, Signal::Pd6);
        assert!(blink.functions.contains(&"GPIO"));
    }

    #[test]
    fn matches_v1004_r1_svg_power_and_nc_pins() {
        let pinout = ONE_DOLLAR_BOARD_PINOUT;

        assert_eq!(pinout.pin(10).map(|pin| pin.signal), Some(Signal::Power3v3));
        assert_eq!(pinout.pin(11).map(|pin| pin.signal), Some(Signal::Ground));
        assert_eq!(
            pinout.pin(16).map(|pin| pin.kind),
            Some(PinKind::NotConnected)
        );
        assert_eq!(
            pinout.pin(17).map(|pin| pin.kind),
            Some(PinKind::NotConnected)
        );
        assert_eq!(
            pinout.pin(18).map(|pin| pin.kind),
            Some(PinKind::NotConnected)
        );
    }
}
