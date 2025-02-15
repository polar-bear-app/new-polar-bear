use eframe::egui::Key;

/// Transformation from Egui's Key to Winit's KeyCode: https://github.com/emilk/egui/blob/08c5a641a17580fb6cfac947aaf95634018abeb7/crates/egui-winit/src/lib.rs#L1148
/// Transformation from Winit's KeyCode to scan code: https://github.com/rust-windowing/winit/blob/c09160d1a8cb6bb560cce32b72d6c0b0b673c9dc/src/platform_impl/linux/common/xkb/keymap.rs#L292
///
/// In the future, if Egui support forwarding the original scan code, we can remove this.
pub fn to_scan_code(key: Key) -> Option<u32> {
    match key {
        Key::ArrowDown => Some(108),
        Key::ArrowLeft => Some(105),
        Key::ArrowRight => Some(106),
        Key::ArrowUp => Some(103),
        Key::Escape => Some(1),
        Key::Tab => Some(15),
        Key::Backspace => Some(14),
        Key::Enter => Some(28), // Main Enter key
        Key::Insert => Some(110),
        Key::Delete => Some(111),
        Key::Home => Some(102),
        Key::End => Some(107),
        Key::PageUp => Some(104),
        Key::PageDown => Some(109),
        Key::Space => Some(57),
        Key::Comma => Some(51),
        Key::Period => Some(52),
        Key::Semicolon => Some(39),
        Key::Backslash => Some(43),
        Key::Slash => Some(53), // Main slash
        Key::OpenBracket => Some(26),
        Key::CloseBracket => Some(27),
        Key::Backtick => Some(41),
        Key::Quote => Some(40),
        Key::Cut => Some(137),
        Key::Copy => Some(133),
        Key::Paste => Some(135),
        Key::Minus => Some(12), // Main minus
        Key::Plus => Some(78),  // NumpadAdd
        Key::Equals => Some(13),
        Key::Num0 => Some(11), // Digit0
        Key::Num1 => Some(2),
        Key::Num2 => Some(3),
        Key::Num3 => Some(4),
        Key::Num4 => Some(5),
        Key::Num5 => Some(6),
        Key::Num6 => Some(7),
        Key::Num7 => Some(8),
        Key::Num8 => Some(9),
        Key::Num9 => Some(10),
        Key::A => Some(30),
        Key::B => Some(48),
        Key::C => Some(46),
        Key::D => Some(32),
        Key::E => Some(18),
        Key::F => Some(33),
        Key::G => Some(34),
        Key::H => Some(35),
        Key::I => Some(23),
        Key::J => Some(36),
        Key::K => Some(37),
        Key::L => Some(38),
        Key::M => Some(50),
        Key::N => Some(49),
        Key::O => Some(24),
        Key::P => Some(25),
        Key::Q => Some(16),
        Key::R => Some(19),
        Key::S => Some(31),
        Key::T => Some(20),
        Key::U => Some(22),
        Key::V => Some(47),
        Key::W => Some(17),
        Key::X => Some(45),
        Key::Y => Some(21),
        Key::Z => Some(44),
        Key::F1 => Some(59),
        Key::F2 => Some(60),
        Key::F3 => Some(61),
        Key::F4 => Some(62),
        Key::F5 => Some(63),
        Key::F6 => Some(64),
        Key::F7 => Some(65),
        Key::F8 => Some(66),
        Key::F9 => Some(67),
        Key::F10 => Some(68),
        Key::F11 => Some(87),
        Key::F12 => Some(88),
        Key::F13 => Some(183),
        Key::F14 => Some(184),
        Key::F15 => Some(185),
        Key::F16 => Some(186),
        Key::F17 => Some(187),
        Key::F18 => Some(188),
        Key::F19 => Some(189),
        Key::F20 => Some(190),
        Key::F21 => Some(191),
        Key::F22 => Some(192),
        Key::F23 => Some(193),
        Key::F24 => Some(194),
        // F25-F35 and other unmapped keys return None
        _ => None,
    }
}
