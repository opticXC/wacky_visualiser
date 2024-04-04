use raylib_ffi;

pub struct Theme {
    pub background: raylib_ffi::Color,
    pub foreground: raylib_ffi::Color,
    pub accent: raylib_ffi::Color,
    pub text: raylib_ffi::Color,
}

// catppuccin

pub mod catppuccin {
    pub enum Accents {
        Rosewater,
        Flamingo,
        Pink,
        Mauve,
        Red,
        Maroon,
        Peach,
        Yellow,
        Green,
        Teal,
        Sky,
        Saphire,
        Blue,
        Lavender,
    }

    pub fn mocha(accent: Accents) -> super::Theme {
        let accent_color = match accent {
            Accents::Rosewater => raylib_ffi::Color {
                r: 245,
                g: 244,
                b: 220,
                a: 255,
            },
            Accents::Flamingo => raylib_ffi::Color {
                r: 242,
                g: 205,
                b: 205,
                a: 255,
            },
            Accents::Pink => raylib_ffi::Color {
                r: 245,
                g: 194,
                b: 231,
                a: 255,
            },
            Accents::Mauve => raylib_ffi::Color {
                r: 203,
                g: 166,
                b: 247,
                a: 255,
            },
            Accents::Red => raylib_ffi::Color {
                r: 243,
                g: 166,
                b: 247,
                a: 255,
            },
            Accents::Maroon => raylib_ffi::Color {
                r: 235,
                g: 160,
                b: 172,
                a: 255,
            },
            Accents::Peach => raylib_ffi::Color {
                r: 250,
                g: 229,
                b: 153,
                a: 255,
            },
            Accents::Yellow => raylib_ffi::Color {
                r: 249,
                g: 226,
                b: 175,
                a: 255,
            },
            Accents::Green => raylib_ffi::Color {
                r: 166,
                g: 227,
                b: 161,
                a: 255,
            },
            Accents::Teal => raylib_ffi::Color {
                r: 148,
                g: 226,
                b: 213,
                a: 255,
            },
            Accents::Sky => raylib_ffi::Color {
                r: 137,
                g: 220,
                b: 235,
                a: 255,
            },
            Accents::Saphire => raylib_ffi::Color {
                r: 116,
                g: 199,
                b: 236,
                a: 255,
            },
            Accents::Blue => raylib_ffi::Color {
                r: 137,
                g: 180,
                b: 250,
                a: 255,
            },
            Accents::Lavender => raylib_ffi::Color {
                r: 180,
                g: 190,
                b: 254,
                a: 255,
            },
        };

        super::Theme {
            background: raylib_ffi::Color {
                r: 17,
                g: 17,
                b: 27,
                a: 255,
            },
            foreground: raylib_ffi::Color {
                r: 30,
                g: 30,
                b: 46,
                a: 255,
            },
            accent: accent_color,
            text: raylib_ffi::Color {
                r: 205,
                g: 214,
                b: 244,
                a: 255,
            },
        }
    }
}
