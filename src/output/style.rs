use std::fmt;

pub struct StyledText {
    text: String,
    foreground: Option<Color>,
    background: Option<Color>,
    style: Option<Style>,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Style {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
}

pub struct ThemeColors;

impl ThemeColors {
    pub const HEADER: Color = Color::Cyan;
    pub const SEPARATOR: Color = Color::Cyan;
    pub const TABLE_HEADER: Color = Color::Blue;
    pub const LABEL: Color = Color::White;
    pub const NUMBER: Color = Color::Green;
    pub const LANGUAGE: Color = Color::White;
}

impl StyledText {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            foreground: None,
            background: None,
            style: None,
        }
    }

    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    #[allow(dead_code)]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
    
}

impl fmt::Display for StyledText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut codes = Vec::new();

        if let Some(style) = &self.style {
            codes.push(style.to_ansi_code());
        }

        if let Some(color) = &self.foreground {
            codes.push(color.to_foreground_ansi_code());
        }

        if let Some(color) = &self.background {
            codes.push(color.to_background_ansi_code());
        }

        if codes.is_empty() {
            write!(f, "{}", self.text)
        } else {
            write!(f, "\x1B[{}m{}\x1B[0m", codes.join(";"), self.text)
        }
    }
}

impl Color {
    fn to_foreground_ansi_code(&self) -> String {
        (match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
        })
        .to_string()
    }

    fn to_background_ansi_code(&self) -> String {
        (match self {
            Color::Black => 40,
            Color::Red => 41,
            Color::Green => 42,
            Color::Yellow => 43,
            Color::Blue => 44,
            Color::Magenta => 45,
            Color::Cyan => 46,
            Color::White => 47,
            Color::BrightBlack => 100,
            Color::BrightRed => 101,
            Color::BrightGreen => 102,
            Color::BrightYellow => 103,
            Color::BrightBlue => 104,
            Color::BrightMagenta => 105,
            Color::BrightCyan => 106,
            Color::BrightWhite => 107,
        })
        .to_string()
    }
}

impl Style {
    fn to_ansi_code(&self) -> String {
        (match self {
            Style::Bold => 1,
            Style::Dim => 2,
            Style::Italic => 3,
            Style::Underline => 4,
            Style::Blink => 5,
            Style::Reverse => 7,
        })
        .to_string()
    }
}

pub fn header(text: &str) -> StyledText {
    StyledText::new(text)
        .foreground(ThemeColors::HEADER)
        .style(Style::Bold)
}

pub fn success(text: &str) -> StyledText {
    StyledText::new(text).foreground(Color::Green)
}

pub fn warning(text: &str) -> StyledText {
    StyledText::new(text).foreground(Color::Yellow)
}

pub fn error(text: &str) -> StyledText {
    StyledText::new(text).foreground(Color::Red)
}

pub fn info(text: &str) -> StyledText {
    StyledText::new(text).foreground(Color::Blue)
}

pub fn highlight(text: &str) -> StyledText {
    StyledText::new(text)
        .foreground(ThemeColors::LABEL)
        .style(Style::Bold)
}

pub fn bold(text: &str) -> StyledText {
    StyledText::new(text)
        .style(Style::Bold)
}

pub fn dimmed(text: &str) -> StyledText {
    StyledText::new(text)
        .style(Style::Dim)
}

pub fn print_header(text: &str) {
    println!("{}", header(text));
}

pub fn print_success(text: &str) {
    println!("{}", success(text));
}

pub fn print_warning(text: &str) {
    println!("{}", warning(text));
}

pub fn print_error(text: &str) {
    println!("{}", error(text));
}

pub fn print_info(text: &str) {
    println!("{}", info(text));
}

#[allow(dead_code)]
pub fn print_highlight(text: &str) {
    println!("{}", highlight(text));
}
