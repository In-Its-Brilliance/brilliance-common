use std::borrow::Cow;

use log::Level;

const COLORS_REGEX: &str = r"(?:&[0-9a-z]{1})";

// https://minecraft.fandom.com/wiki/Formatting_codes

pub enum Color {
    Reset,
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,

    Violet,
    Magenta,
    Pink,
    Coral,
    Orange,
}

impl Color {
    pub fn from_str(origin: &str) -> Option<Color> {
        let color = match origin {
            "&r" => Color::Reset,
            "&0" => Color::Black,
            "&1" => Color::DarkBlue,
            "&2" => Color::DarkGreen,
            "&3" => Color::DarkAqua,
            "&4" => Color::DarkRed,
            "&5" => Color::DarkPurple,
            "&6" => Color::Gold,
            "&7" => Color::Gray,
            "&8" => Color::DarkGray,
            "&9" => Color::Blue,
            "&a" => Color::Green,
            "&b" => Color::Aqua,
            "&c" => Color::Red,
            "&d" => Color::LightPurple,
            "&e" => Color::Yellow,
            "&f" => Color::White,

            "&g" => Color::Violet,
            "&l" => Color::Magenta,
            "&m" => Color::Pink,
            "&o" => Color::Coral,
            "&s" => Color::Orange,
            _ => return None,
        };
        Some(color)
    }

    pub fn to_terminal_code(&self) -> Cow<'static, str> {
        match *self {
            Color::Reset => "0".into(),
            Color::Black => "38;2;0;0;0".into(),
            Color::DarkBlue => "38;2;0;0;170".into(),
            Color::DarkGreen => "38;2;0;170;0".into(),
            Color::DarkAqua => "38;2;0;170;170".into(),
            Color::DarkRed => "38;2;170;0;0".into(),
            Color::DarkPurple => "38;2;170;0;170".into(),
            Color::Gold => "38;2;255;170;0".into(),
            Color::Gray => "38;2;170;170;170".into(),
            Color::DarkGray => "38;2;120;120;120".into(),
            Color::Blue => "38;2;85;85;255".into(),
            Color::Green => "38;2;85;255;85".into(),
            Color::Aqua => "38;2;85;255;255".into(),
            Color::Red => "38;2;255;85;85".into(),
            Color::LightPurple => "38;2;255;85;255".into(),
            Color::Yellow => "38;2;255;255;85".into(),
            Color::White => "38;2;255;255;255".into(),

            Color::Violet => "38;2;138;92;246".into(),
            Color::Magenta => "38;2;168;85;210".into(),
            Color::Pink => "38;2;200;100;170".into(),
            Color::Coral => "38;2;235;130;120".into(),
            Color::Orange => "38;2;255;170;80".into(),
        }
    }

    pub fn to_terminal(&self) -> String {
        format!("\x1b[0;{}m", self.to_terminal_code())
    }

    pub fn to_godot_tag(&self) -> Cow<'static, str> {
        match *self {
            Color::Reset => "[/color]".into(),
            Color::Black => "[color=#000000]".into(),
            Color::DarkBlue => "[color=#0000AA]".into(),
            Color::DarkGreen => "[color=#00AA00]".into(),
            Color::DarkAqua => "[color=#00AAAA]".into(),
            Color::DarkRed => "[color=#AA0000]".into(),
            Color::DarkPurple => "[color=#AA00AA]".into(),
            Color::Gold => "[color=#FFAA00]".into(),
            Color::Gray => "[color=#AAAAAA]".into(),
            Color::DarkGray => "[color=#787878]".into(),
            Color::Blue => "[color=#5555FF]".into(),
            Color::Green => "[color=#55FF55]".into(),
            Color::Aqua => "[color=#55FFFF]".into(),
            Color::Red => "[color=#FF5555]".into(),
            Color::LightPurple => "[color=#FF55FF]".into(),
            Color::Yellow => "[color=#FFFF55]".into(),
            Color::White => "[color=#FFFFFF]".into(),

            Color::Violet => "[color=#8A5CF6]".into(),
            Color::Magenta => "[color=#A855D2]".into(),
            Color::Pink => "[color=#C864AA]".into(),
            Color::Coral => "[color=#EB8278]".into(),
            Color::Orange => "[color=#FFAA50]".into(),
        }
    }
}

pub fn parse_to_terminal_colors(origin: &String) -> String {
    let mut result = origin.clone();
    let re = regex::Regex::new(COLORS_REGEX).unwrap();

    let mut offset = 0;
    for c in re.find_iter(&origin) {
        if c.start() + offset >= 1 {
            let pre = result.as_bytes()[c.start() - 1 + offset] as char;
            if pre == '\\' {
                result.remove(c.start() - 1 + offset);
                offset -= 1;
                continue;
            }
        }

        let replace_str = match Color::from_str(c.as_str()) {
            Some(c) => c.to_terminal(),
            None => continue,
        };
        result.replace_range(c.start() + offset..c.end() + offset, &replace_str);
        offset += replace_str.len() - c.as_str().len();
    }
    format!("{}{}", result, Color::Reset.to_terminal())
}

pub fn parse_to_console_godot(origin: &String) -> String {
    let mut result = origin.clone();
    let re = regex::Regex::new(COLORS_REGEX).unwrap();

    let mut offset = 0;
    for c in re.find_iter(&origin) {
        if c.start() + offset >= 1 {
            let pre = result.as_bytes()[c.start() - 1 + offset] as char;
            if pre == '\\' {
                result.remove(c.start() - 1 + offset);
                offset -= 1;
                continue;
            }
        }

        let replace_str = match Color::from_str(c.as_str()) {
            Some(c) => c.to_godot_tag(),
            None => continue,
        };
        result.replace_range(c.start() + offset..c.end() + offset, &replace_str);
        offset += replace_str.len() - c.as_str().len();
    }
    if result.find("[color").is_some() {
        result = format!("{}{}", result, Color::Reset.to_godot_tag());
    }
    result
}

pub fn get_log_level_color(level: &Level) -> Cow<'static, str> {
    match level {
        Level::Error => "&c".into(),
        Level::Warn => "&6".into(),
        Level::Info => "&a".into(),
        Level::Debug => "&e".into(),
        Level::Trace => "&8".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::colors::parse_to_console_godot;

    use super::parse_to_terminal_colors;

    #[test]
    fn test_terminal_colors() {
        let r = parse_to_terminal_colors(&"&5magenta_blue-&1_skeep-\\&2_gold-&6_red-&4_test".to_string());
        assert_eq!(
            r,
            "\u{1b}[0;38;2;170;0;170mmagenta_blue-\u{1b}[0;38;2;0;0;170m_skeep-&2_gold-\u{1b}[0;38;2;255;170;0m_red-\u{1b}[0;38;2;170;0;0m_test\u{1b}[0;0m"
                .to_string()
        );
    }

    #[test]
    fn test_to_godot() {
        let r = parse_to_console_godot(&"time: &8main &aINFO&r: text".to_string());
        assert_eq!(
            r,
            "time: [color=#787878]main [color=#55FF55]INFO[/color]: text[/color]".to_string()
        );
    }
}
