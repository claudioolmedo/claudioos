#[path = "claudioos/board_image.rs"]
mod board_image;

use std::env;
use std::fmt::Write;
use std::fs;
use std::io::Cursor;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use board_image::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_PNG, BOARD_IMAGE_WIDTH};
use claudioos::{
    BoardBus, BoardTarget, EventKind, Machine, PinKind, RamBus, RunLimit, Signal, StopReason,
    ONE_DOLLAR_BOARD_PINOUT,
};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const LUI_X1_GPIO: u32 = 0x4001_10b7;
const ADDI_X2_ONE: u32 = 0x0010_0113;
const SW_X2_GPIO_OUT: u32 = 0x0020_a623;
const SW_ZERO_GPIO_OUT: u32 = 0x0000_a623;
const EBREAK: u32 = 0x0010_0073;
const DIGITAL_PIN_COUNT: usize = 15;
const GPIO_ON_COLOR: u32 = 0xf2a900;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PinIndicator {
    signal: Signal,
    label: &'static str,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    bit: u8,
}

const PIN_INDICATORS: [PinIndicator; DIGITAL_PIN_COUNT] = [
    PinIndicator {
        signal: Signal::Pc0,
        label: "0 PC0",
        x: 33,
        y: 255,
        w: 14,
        h: 15,
        bit: 0,
    },
    PinIndicator {
        signal: Signal::Pc1,
        label: "1 PC1",
        x: 33,
        y: 294,
        w: 14,
        h: 15,
        bit: 1,
    },
    PinIndicator {
        signal: Signal::Pc2,
        label: "2 PC2",
        x: 33,
        y: 333,
        w: 14,
        h: 14,
        bit: 2,
    },
    PinIndicator {
        signal: Signal::Pc3,
        label: "3 PC3",
        x: 33,
        y: 371,
        w: 14,
        h: 15,
        bit: 3,
    },
    PinIndicator {
        signal: Signal::Pc4,
        label: "4 PC4",
        x: 33,
        y: 410,
        w: 14,
        h: 15,
        bit: 4,
    },
    PinIndicator {
        signal: Signal::Pc5,
        label: "5 PC5",
        x: 33,
        y: 449,
        w: 14,
        h: 14,
        bit: 5,
    },
    PinIndicator {
        signal: Signal::Pc6,
        label: "6 PC6",
        x: 33,
        y: 487,
        w: 14,
        h: 15,
        bit: 6,
    },
    PinIndicator {
        signal: Signal::Pc7,
        label: "7 PC7",
        x: 33,
        y: 526,
        w: 14,
        h: 14,
        bit: 7,
    },
    PinIndicator {
        signal: Signal::Pa1,
        label: "8 PA1",
        x: 33,
        y: 564,
        w: 14,
        h: 15,
        bit: 1,
    },
    PinIndicator {
        signal: Signal::Pa2,
        label: "9 PA2",
        x: 33,
        y: 603,
        w: 14,
        h: 15,
        bit: 2,
    },
    PinIndicator {
        signal: Signal::Pd1,
        label: "12 PD1",
        x: 342,
        y: 333,
        w: 15,
        h: 14,
        bit: 1,
    },
    PinIndicator {
        signal: Signal::Pd7,
        label: "13 PD7",
        x: 342,
        y: 371,
        w: 15,
        h: 15,
        bit: 7,
    },
    PinIndicator {
        signal: Signal::Pd0,
        label: "14 PD0",
        x: 342,
        y: 410,
        w: 15,
        h: 14,
        bit: 0,
    },
    PinIndicator {
        signal: Signal::Pd2,
        label: "15 PD2",
        x: 342,
        y: 449,
        w: 15,
        h: 15,
        bit: 2,
    },
    PinIndicator {
        signal: Signal::Pd6,
        label: "19 PD6",
        x: 342,
        y: 603,
        w: 15,
        h: 15,
        bit: 6,
    },
];

type PinFrame = [bool; DIGITAL_PIN_COUNT];

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let command = args.get(1).map(String::as_str);

    match command {
        Some("pinout") => {
            print_pinout();
            ExitCode::SUCCESS
        }
        Some("blink") => run_blink(),
        Some("visual-blink") => run_visual_blink(),
        Some("target") => {
            print_target();
            ExitCode::SUCCESS
        }
        Some("rv32ec-onedollarboard") => match args.get(2).map(String::as_str) {
            Some("run") => match args.get(3) {
                Some(path) => run_real_binary(path),
                None => {
                    eprintln!("missing binary path");
                    print_help();
                    ExitCode::FAILURE
                }
            },
            Some("visual") => match args.get(3) {
                Some(path) => run_visual_binary(path),
                None => {
                    eprintln!("missing binary path");
                    print_help();
                    ExitCode::FAILURE
                }
            },
            Some("model") => {
                print_model();
                ExitCode::SUCCESS
            }
            _ => {
                print_help();
                ExitCode::FAILURE
            }
        },
        Some("visual-bin") => match args.get(2) {
            Some(path) => run_visual_binary(path),
            None => {
                eprintln!("missing binary path");
                print_help();
                ExitCode::FAILURE
            }
        },
        Some("run-bin") => match args.get(2) {
            Some(path) => run_real_binary(path),
            None => {
                eprintln!("missing binary path");
                print_help();
                ExitCode::FAILURE
            }
        },
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(_) => {
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("Claudio OS");
    println!();
    println!("Commands:");
    println!("  pinout   Show the One Dollar Board pinout model");
    println!("  target   Show the stable board target contract");
    println!("  blink    Compile-view and emulate the board blink test");
    println!("  visual-blink");
    println!("           Open a native window and show the emulated LED blinking");
    println!("  visual-bin <path>");
    println!("           Open a native window and blink from a raw board binary trace");
    println!("  run-bin <path>");
    println!("           Load a raw board binary and trace GPIO writes");
    println!("  rv32ec-onedollarboard run <path>");
    println!("           Run a raw binary against the board compatibility target");
    println!("  rv32ec-onedollarboard visual <path>");
    println!("           Visualize LED activity from the board compatibility target");
    println!("  rv32ec-onedollarboard model");
    println!("           Show the One Dollar Board 1.004 model contract");
}

fn print_target() {
    let target = BoardTarget::RV32EC_ONE_DOLLAR_BOARD;

    println!("{}", target.name);
    println!("cli: {}", target.cli_name);
    println!("model: {} {}", target.model.name, target.model.revision);
    println!("family: {}", target.model.compatibility_family);
    println!("isa: {}", target.instruction_set.name());
    println!("flash: {} bytes", target.flash_bytes);
    println!("ram: {} bytes", target.ram_bytes);
    println!(
        "portable rust programs: {}",
        target.keeps_programs_portable()
    );
}

fn print_model() {
    let target = BoardTarget::RV32EC_ONE_DOLLAR_BOARD;
    let pinout = ONE_DOLLAR_BOARD_PINOUT;

    println!("{} {}", target.model.name, target.model.revision);
    println!("target: {}", target.name);
    println!("family: {}", target.model.compatibility_family);
    println!("connector: {}", pinout.connector_name);
    println!("auxiliary: {}", pinout.auxiliary_connector_name);
    println!("blink pin: {}", pinout.blink_pin);
    println!("pins: {}", pinout.pins.len());
}

fn print_pinout() {
    let pinout = ONE_DOLLAR_BOARD_PINOUT;

    println!("{} {}", pinout.board_name, pinout.revision);
    println!(
        "{} / {}",
        pinout.connector_name, pinout.auxiliary_connector_name
    );
    println!();
    println!(" left header              right header");
    println!(" +------------------+     +------------------+");

    for row in 0..10 {
        let left = pinout.pin(row as u8);
        let right = pinout.pin((10 + row) as u8);
        println!(" | {} |     | {} |", pin_cell(left), pin_cell(right));
    }

    println!(" +------------------+     +------------------+");
    println!();

    for pin in pinout.pins {
        println!(
            "{:>2} {:<4} {:<7} {:<6} {}",
            pin.number,
            pin.signal.name(),
            kind_name(pin.kind),
            pin.rj_pin.unwrap_or("-"),
            join_functions(pin.functions)
        );
    }
}

fn run_blink() -> ExitCode {
    let pinout = ONE_DOLLAR_BOARD_PINOUT;
    let blink = pinout.blink_pin();

    println!("Claudio OS compile view");
    println!("target board: {}", pinout.board_name);
    println!(
        "target architecture: {}",
        BoardTarget::RV32EC_ONE_DOLLAR_BOARD.instruction_set.name()
    );
    println!(
        "blink signal: pin {} / {}",
        blink.number,
        blink.signal.name()
    );
    println!();
    println!("[1/4] build host tool: Rust executable for this Mac");
    println!("[2/4] lower blink test: built-in RV32E instruction image");
    println!("[3/4] load image: Claudio OS emulator RAM");
    println!("[4/4] run: GPIO trace");
    println!();

    let mut bus = RamBus::<8>::new();
    let program = [
        LUI_X1_GPIO,
        ADDI_X2_ONE,
        SW_X2_GPIO_OUT,
        SW_ZERO_GPIO_OUT,
        EBREAK,
    ];

    for (index, word) in program.iter().copied().enumerate() {
        if let Err(fault) = bus.load_word(index, word) {
            eprintln!("load failed: {:?}", fault);
            return ExitCode::FAILURE;
        }
    }

    let mut machine = Machine::<_, 8>::new(bus);
    let reason = machine.run(RunLimit { max_cycles: 16 });

    for event in machine.events() {
        match event.kind {
            EventKind::GpioWrite => {
                let state = if event.value == 0 { "off" } else { "on" };
                let led = if event.value == 0 { "." } else { "*" };
                println!(
                    "cycle {:>2}: {} {} {}",
                    event.cycle,
                    blink.signal.name(),
                    state,
                    led
                );
            }
        }
    }

    println!();
    println!("stop: {:?}", reason);
    println!("cycles: {}", machine.cycles());

    if reason == StopReason::Ebreak {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_visual_blink() -> ExitCode {
    let pinout = ONE_DOLLAR_BOARD_PINOUT;
    let blink = pinout.blink_pin();
    let frames = blink_pin_frames(8);
    let base_image = match decode_board_image() {
        Ok(image) => image,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    if frames.is_empty() {
        eprintln!("blink trace did not produce GPIO events");
        return ExitCode::FAILURE;
    }

    println!("Claudio OS visual blink");
    println!(
        "{} {} / pin {} {}",
        pinout.board_name,
        pinout.revision,
        blink.number,
        blink.signal.name()
    );
    println!("close the window or press Escape to stop");

    let mut window = match open_visual_window("Claudio OS - One Dollar Board blink") {
        Ok(window) => window,
        Err(error) => {
            eprintln!("window failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut buffer = base_image.clone();

    for frame in frames.iter().copied() {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            return ExitCode::SUCCESS;
        }

        buffer.copy_from_slice(&base_image);
        draw_pinout_overlay(&mut buffer, &frame);

        if let Err(error) =
            window.update_with_buffer(&buffer, BOARD_IMAGE_WIDTH, BOARD_IMAGE_HEIGHT)
        {
            eprintln!("window update failed: {error}");
            return ExitCode::FAILURE;
        }

        thread::sleep(Duration::from_millis(450));
    }

    for _ in 0..8 {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        buffer.copy_from_slice(&base_image);
        draw_pinout_overlay(&mut buffer, &[false; DIGITAL_PIN_COUNT]);

        if let Err(error) =
            window.update_with_buffer(&buffer, BOARD_IMAGE_WIDTH, BOARD_IMAGE_HEIGHT)
        {
            eprintln!("window update failed: {error}");
            return ExitCode::FAILURE;
        }

        thread::sleep(Duration::from_millis(100));
    }

    ExitCode::SUCCESS
}

fn run_real_binary(path: &str) -> ExitCode {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut bus = BoardBus::<65536, 4096>::new();
    if let Err(fault) = bus.load_flash(&bytes) {
        eprintln!("failed to load binary: {:?}", fault);
        return ExitCode::FAILURE;
    }

    let mut machine = Machine::<_, 32>::new(bus);
    if let Ok(raw_steps) = env::var("CLAUDIOOS_TRACE_STEPS") {
        let steps = raw_steps.parse::<usize>().unwrap_or(0);
        for _ in 0..steps {
            let pc = machine.pc();
            let reason = machine.step();
            println!("trace pc=0x{pc:08x} reason={reason:?}");
            if reason != StopReason::Running {
                break;
            }
        }
    }
    let reason = machine.run(RunLimit {
        max_cycles: 2_000_000,
    });
    let events = machine.events().collect::<Vec<_>>();

    println!("Claudio OS binary run");
    println!("target: {}", BoardTarget::RV32EC_ONE_DOLLAR_BOARD.name);
    println!("binary: {path}");
    println!("size: {} bytes", bytes.len());
    println!("stop: {:?}", reason);
    println!("pc: 0x{:08x}", machine.pc());
    println!("cycles: {}", machine.cycles());
    println!();

    if events.is_empty() {
        println!("gpio events: none");
        return ExitCode::FAILURE;
    }

    println!("gpio events:");
    for event in events {
        let port = match event.address {
            0x4001_0810 => "GPIOA",
            0x4001_1010 => "GPIOC",
            0x4001_1410 => "GPIOD",
            _ => "GPIO",
        };
        let action = if event.value & 0xffff != 0 {
            "set"
        } else if event.value >> 16 != 0 {
            "reset"
        } else {
            "write"
        };
        println!(
            "cycle {:>8}: {} {} value=0x{:08x}",
            event.cycle, port, action, event.value
        );
    }

    ExitCode::SUCCESS
}

fn run_visual_binary(path: &str) -> ExitCode {
    let frames = match real_binary_pin_frames(path) {
        Ok(frames) => frames,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let base_image = match decode_board_image() {
        Ok(image) => image,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    if frames.is_empty() {
        eprintln!("binary trace did not produce GPIO events");
        return ExitCode::FAILURE;
    }

    println!("Claudio OS visual binary");
    println!("target: {}", BoardTarget::RV32EC_ONE_DOLLAR_BOARD.name);
    println!("binary: {path}");
    println!("close the window or press Escape to stop");

    let mut window = match open_visual_window("Claudio OS - real binary blink") {
        Ok(window) => window,
        Err(error) => {
            eprintln!("window failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut buffer = base_image.clone();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for frame in frames.iter().copied() {
            if !window.is_open() || window.is_key_down(Key::Escape) {
                return ExitCode::SUCCESS;
            }

            buffer.copy_from_slice(&base_image);
            draw_pinout_overlay(&mut buffer, &frame);

            if let Err(error) =
                window.update_with_buffer(&buffer, BOARD_IMAGE_WIDTH, BOARD_IMAGE_HEIGHT)
            {
                eprintln!("window update failed: {error}");
                return ExitCode::FAILURE;
            }

            thread::sleep(Duration::from_millis(450));
        }
    }

    ExitCode::SUCCESS
}

fn open_visual_window(title: &str) -> Result<Window, minifb::Error> {
    let mut window = Window::new(
        title,
        BOARD_IMAGE_WIDTH,
        BOARD_IMAGE_HEIGHT,
        WindowOptions {
            borderless: true,
            title: false,
            resize: true,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: true,
            ..WindowOptions::default()
        },
    )?;
    window.set_position(0, 0);
    window.set_background_color(247, 243, 232);
    window.set_target_fps(30);
    Ok(window)
}

fn real_binary_pin_frames(path: &str) -> Result<Vec<PinFrame>, String> {
    let bytes = fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let mut bus = BoardBus::<65536, 4096>::new();
    bus.load_flash(&bytes)
        .map_err(|fault| format!("failed to load binary: {fault:?}"))?;

    let mut machine = Machine::<_, 256>::new(bus);
    let reason = machine.run(RunLimit { max_cycles: 80_000 });
    let frames = pin_frames_from_events(machine.events());

    if frames.is_empty() {
        return Err(format!(
            "no GPIO events found; stop={reason:?} pc=0x{:08x} cycles={}",
            machine.pc(),
            machine.cycles()
        ));
    }

    Ok(frames)
}

fn decode_board_image() -> Result<Vec<u32>, String> {
    let decoder = png::Decoder::new(Cursor::new(BOARD_IMAGE_PNG));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("png decode failed: {error}"))?;
    let mut bytes = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut bytes)
        .map_err(|error| format!("png frame failed: {error}"))?;

    if info.width as usize != BOARD_IMAGE_WIDTH || info.height as usize != BOARD_IMAGE_HEIGHT {
        return Err("embedded board image has unexpected dimensions".to_owned());
    }

    match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => {
            Ok(rgba_to_minifb_pixels(&bytes[..info.buffer_size()]))
        }
        _ => Err("embedded board image must be 8-bit RGBA PNG".to_owned()),
    }
}

fn rgba_to_minifb_pixels(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|pixel| {
            let alpha = pixel[3] as u32;
            let inverse = 255 - alpha;
            let r = (pixel[0] as u32 * alpha + 255 * inverse) / 255;
            let g = (pixel[1] as u32 * alpha + 255 * inverse) / 255;
            let b = (pixel[2] as u32 * alpha + 255 * inverse) / 255;
            (r << 16) | (g << 8) | b
        })
        .collect()
}

fn draw_pinout_overlay(buffer: &mut [u32], frame: &PinFrame) {
    for (index, indicator) in PIN_INDICATORS.iter().enumerate() {
        let active = frame[index];
        let fill = if active { GPIO_ON_COLOR } else { 0x050505 };
        let edge = if active { 0x050505 } else { 0xffffff };
        draw_rect(
            buffer,
            BOARD_IMAGE_WIDTH,
            indicator.x,
            indicator.y,
            indicator.w,
            indicator.h,
            edge,
        );
        draw_rect(
            buffer,
            BOARD_IMAGE_WIDTH,
            indicator.x + 2,
            indicator.y + 2,
            indicator.w.saturating_sub(4),
            indicator.h.saturating_sub(4),
            fill,
        );
        draw_pin_label(
            buffer,
            indicator.x + indicator.w + 6,
            indicator.y + 2,
            indicator.label,
            active,
        );
    }
}

fn blink_pin_frames(repetitions: usize) -> Vec<PinFrame> {
    let mut frames = Vec::with_capacity(repetitions * 2);

    for _ in 0..repetitions {
        let mut bus = RamBus::<8>::new();
        for (index, word) in [
            LUI_X1_GPIO,
            ADDI_X2_ONE,
            SW_X2_GPIO_OUT,
            SW_ZERO_GPIO_OUT,
            EBREAK,
        ]
        .iter()
        .copied()
        .enumerate()
        {
            if bus.load_word(index, word).is_err() {
                return frames;
            }
        }

        let mut machine = Machine::<_, 8>::new(bus);
        if machine.run(RunLimit { max_cycles: 16 }) != StopReason::Ebreak {
            return frames;
        }

        let blink_index = indicator_index(Signal::Pd6).unwrap_or(DIGITAL_PIN_COUNT - 1);
        for event in machine.events() {
            if event.kind == EventKind::GpioWrite {
                let mut frame = [false; DIGITAL_PIN_COUNT];
                frame[blink_index] = event.value != 0;
                frames.push(frame);
            }
        }
    }

    frames
}

fn pin_frames_from_events(events: impl Iterator<Item = claudioos::Event>) -> Vec<PinFrame> {
    let mut frame = [false; DIGITAL_PIN_COUNT];
    let mut frames = Vec::new();

    for event in events {
        if event.kind != EventKind::GpioWrite {
            continue;
        }

        let Some(port) = gpio_port_from_event_address(event.address) else {
            continue;
        };

        let set = event.value & 0xffff;
        let reset = (event.value >> 16) & 0xffff;
        for (index, indicator) in PIN_INDICATORS.iter().enumerate() {
            if indicator_port(indicator.signal) != Some(port) {
                continue;
            }

            let mask = 1u32 << indicator.bit;
            if set & mask != 0 {
                frame[index] = true;
            }
            if reset & mask != 0 {
                frame[index] = false;
            }
        }

        frames.push(frame);
    }

    frames
}

fn indicator_index(signal: Signal) -> Option<usize> {
    PIN_INDICATORS
        .iter()
        .position(|indicator| indicator.signal == signal)
}

fn gpio_port_from_event_address(address: u32) -> Option<char> {
    match address {
        0x4001_0810 => Some('A'),
        0x4001_1010 => Some('C'),
        0x4001_1410 => Some('D'),
        _ => None,
    }
}

fn indicator_port(signal: Signal) -> Option<char> {
    match signal {
        Signal::Pa1 | Signal::Pa2 => Some('A'),
        Signal::Pc0
        | Signal::Pc1
        | Signal::Pc2
        | Signal::Pc3
        | Signal::Pc4
        | Signal::Pc5
        | Signal::Pc6
        | Signal::Pc7 => Some('C'),
        Signal::Pd0 | Signal::Pd1 | Signal::Pd2 | Signal::Pd6 | Signal::Pd7 => Some('D'),
        _ => None,
    }
}

fn draw_rect(buffer: &mut [u32], width: usize, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for yy in y..y + h {
        for xx in x..x + w {
            if let Some(pixel) = buffer.get_mut(yy * width + xx) {
                *pixel = color;
            }
        }
    }
}

fn draw_pin_label(buffer: &mut [u32], x: usize, y: usize, label: &str, active: bool) {
    let color = if active { GPIO_ON_COLOR } else { 0x050505 };
    let shadow = if active { 0x050505 } else { 0xffffff };
    draw_text(buffer, BOARD_IMAGE_WIDTH, x + 1, y + 1, label, shadow);
    draw_text(buffer, BOARD_IMAGE_WIDTH, x, y, label, color);
}

fn draw_text(buffer: &mut [u32], width: usize, x: usize, y: usize, text: &str, color: u32) {
    let mut cursor = x;
    for ch in text.chars() {
        draw_glyph(buffer, width, cursor, y, ch, color);
        cursor += 5;
    }
}

fn draw_glyph(buffer: &mut [u32], width: usize, x: usize, y: usize, ch: char, color: u32) {
    let glyph = glyph_3x5(ch);
    for (row, bits) in glyph.iter().copied().enumerate() {
        for col in 0..3 {
            if bits & (1 << (2 - col)) != 0 {
                draw_rect(buffer, width, x + col, y + row, 1, 1, color);
            }
        }
    }
}

fn glyph_3x5(ch: char) -> [u8; 5] {
    match ch {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        _ => [0; 5],
    }
}

fn pin_cell(pin: Option<&claudioos::BoardPin>) -> String {
    match pin {
        Some(pin) => {
            let mut cell = String::new();
            let _ = write!(
                cell,
                "{:>2} {:<4} {:<8}",
                pin.number,
                pin.signal.name(),
                kind_name(pin.kind)
            );
            cell
        }
        None => "                  ".to_owned(),
    }
}

fn kind_name(kind: PinKind) -> &'static str {
    match kind {
        PinKind::Digital => "digital",
        PinKind::Power => "power",
        PinKind::Ground => "ground",
        PinKind::NotConnected => "nc",
    }
}

fn join_functions(functions: &[&str]) -> String {
    let mut output = String::new();
    for (index, function) in functions.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(function);
    }
    output
}
