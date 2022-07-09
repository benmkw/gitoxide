use crate::value;
use crate::Color;
use bstr::{BStr, BString};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_space = None;
        if let Some(fg) = self.foreground {
            fg.fmt(f)?;
            write_space = Some(());
        }

        if let Some(bg) = self.background {
            if write_space.take().is_some() {
                write!(f, " ")?;
            }
            bg.fmt(f)?;
            write_space = Some(())
        }

        self.attributes.iter().try_for_each(|attr| {
            if write_space.take().is_some() {
                write!(f, " ")
            } else {
                Ok(())
            }
            .and_then(|_| {
                write_space = Some(());
                attr.fmt(f)
            })
        })
    }
}

fn color_err(input: impl Into<BString>) -> value::Error {
    value::Error::new(
        "Colors are specific color values and their attributes, like 'brightred', or 'blue'",
        input,
    )
}

impl TryFrom<&BStr> for Color {
    type Error = value::Error;

    fn try_from(s: &BStr) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(s).map_err(|err| color_err(s).with_err(err))?;
        enum ColorItem {
            Value(Name),
            Attr(Attribute),
        }

        let items = s.split_whitespace().filter_map(|s| {
            if s.is_empty() {
                return None;
            }

            Some(
                Name::from_str(s)
                    .map(ColorItem::Value)
                    .or_else(|_| Attribute::from_str(s).map(ColorItem::Attr)),
            )
        });

        let mut new_self = Self::default();
        for item in items {
            match item {
                Ok(item) => match item {
                    ColorItem::Value(v) => {
                        if new_self.foreground.is_none() {
                            new_self.foreground = Some(v);
                        } else if new_self.background.is_none() {
                            new_self.background = Some(v);
                        } else {
                            return Err(color_err(s));
                        }
                    }
                    ColorItem::Attr(a) => new_self.attributes.push(a),
                },
                Err(_) => return Err(color_err(s)),
            }
        }

        Ok(new_self)
    }
}

impl TryFrom<Cow<'_, BStr>> for Color {
    type Error = value::Error;

    fn try_from(c: Cow<'_, BStr>) -> Result<Self, Self::Error> {
        Self::try_from(c.as_ref())
    }
}

/// Discriminating enum for [`Color`] values.
///
/// `git-config` supports the eight standard colors, their bright variants, an
/// ANSI color code, or a 24-bit hex value prefixed with an octothorpe.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(missing_docs)]
pub enum Name {
    Normal,
    Default,
    Black,
    BrightBlack,
    Red,
    BrightRed,
    Green,
    BrightGreen,
    Yellow,
    BrightYellow,
    Blue,
    BrightBlue,
    Magenta,
    BrightMagenta,
    Cyan,
    BrightCyan,
    White,
    BrightWhite,
    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Default => write!(f, "default"),
            Self::Black => write!(f, "black"),
            Self::BrightBlack => write!(f, "brightblack"),
            Self::Red => write!(f, "red"),
            Self::BrightRed => write!(f, "brightred"),
            Self::Green => write!(f, "green"),
            Self::BrightGreen => write!(f, "brightgreen"),
            Self::Yellow => write!(f, "yellow"),
            Self::BrightYellow => write!(f, "brightyellow"),
            Self::Blue => write!(f, "blue"),
            Self::BrightBlue => write!(f, "brightblue"),
            Self::Magenta => write!(f, "magenta"),
            Self::BrightMagenta => write!(f, "brightmagenta"),
            Self::Cyan => write!(f, "cyan"),
            Self::BrightCyan => write!(f, "brightcyan"),
            Self::White => write!(f, "white"),
            Self::BrightWhite => write!(f, "brightwhite"),
            Self::Ansi(num) => num.fmt(f),
            Self::Rgb(r, g, b) => write!(f, "#{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for Name {
    type Err = value::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let bright = if let Some(rest) = s.strip_prefix("bright") {
            s = rest;
            true
        } else {
            false
        };

        match s {
            "normal" if !bright => return Ok(Self::Normal),
            "-1" if !bright => return Ok(Self::Normal),
            "normal" if bright => return Err(color_err(s)),
            "default" if !bright => return Ok(Self::Default),
            "default" if bright => return Err(color_err(s)),
            "black" if !bright => return Ok(Self::Black),
            "black" if bright => return Ok(Self::BrightBlack),
            "red" if !bright => return Ok(Self::Red),
            "red" if bright => return Ok(Self::BrightRed),
            "green" if !bright => return Ok(Self::Green),
            "green" if bright => return Ok(Self::BrightGreen),
            "yellow" if !bright => return Ok(Self::Yellow),
            "yellow" if bright => return Ok(Self::BrightYellow),
            "blue" if !bright => return Ok(Self::Blue),
            "blue" if bright => return Ok(Self::BrightBlue),
            "magenta" if !bright => return Ok(Self::Magenta),
            "magenta" if bright => return Ok(Self::BrightMagenta),
            "cyan" if !bright => return Ok(Self::Cyan),
            "cyan" if bright => return Ok(Self::BrightCyan),
            "white" if !bright => return Ok(Self::White),
            "white" if bright => return Ok(Self::BrightWhite),
            _ => (),
        }

        if let Ok(v) = u8::from_str(s) {
            return Ok(Self::Ansi(v));
        }

        if let Some(s) = s.strip_prefix('#') {
            if s.len() == 6 {
                let rgb = (
                    u8::from_str_radix(&s[..2], 16),
                    u8::from_str_radix(&s[2..4], 16),
                    u8::from_str_radix(&s[4..], 16),
                );

                if let (Ok(r), Ok(g), Ok(b)) = rgb {
                    return Ok(Self::Rgb(r, g, b));
                }
            }
        }

        Err(color_err(s))
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = value::Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        Self::from_str(std::str::from_utf8(s).map_err(|err| color_err(s).with_err(err))?)
    }
}

/// Discriminating enum for [`Color`] attributes.
///
/// `git-config` supports modifiers and their negators. The negating color
/// attributes are equivalent to having a `no` or `no-` prefix to the normal
/// variant.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(missing_docs)]
pub enum Attribute {
    Reset,
    Bold,
    NoBold,
    Dim,
    NoDim,
    Ul,
    NoUl,
    Blink,
    NoBlink,
    Reverse,
    NoReverse,
    Italic,
    NoItalic,
    Strike,
    NoStrike,
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reset => write!(f, "reset"),
            Self::Bold => write!(f, "bold"),
            Self::NoBold => write!(f, "nobold"),
            Self::Dim => write!(f, "dim"),
            Self::NoDim => write!(f, "nodim"),
            Self::Ul => write!(f, "ul"),
            Self::NoUl => write!(f, "noul"),
            Self::Blink => write!(f, "blink"),
            Self::NoBlink => write!(f, "noblink"),
            Self::Reverse => write!(f, "reverse"),
            Self::NoReverse => write!(f, "noreverse"),
            Self::Italic => write!(f, "italic"),
            Self::NoItalic => write!(f, "noitalic"),
            Self::Strike => write!(f, "strike"),
            Self::NoStrike => write!(f, "nostrike"),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for Attribute {
    type Err = value::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let inverted = if let Some(rest) = s.strip_prefix("no-").or_else(|| s.strip_prefix("no")) {
            s = rest;
            true
        } else {
            false
        };

        match s {
            "reset" if !inverted => Ok(Self::Reset),
            "reset" if inverted => Err(color_err(s)),
            "bold" if !inverted => Ok(Self::Bold),
            "bold" if inverted => Ok(Self::NoBold),
            "dim" if !inverted => Ok(Self::Dim),
            "dim" if inverted => Ok(Self::NoDim),
            "ul" if !inverted => Ok(Self::Ul),
            "ul" if inverted => Ok(Self::NoUl),
            "blink" if !inverted => Ok(Self::Blink),
            "blink" if inverted => Ok(Self::NoBlink),
            "reverse" if !inverted => Ok(Self::Reverse),
            "reverse" if inverted => Ok(Self::NoReverse),
            "italic" if !inverted => Ok(Self::Italic),
            "italic" if inverted => Ok(Self::NoItalic),
            "strike" if !inverted => Ok(Self::Strike),
            "strike" if inverted => Ok(Self::NoStrike),
            _ => Err(color_err(s)),
        }
    }
}

impl TryFrom<&[u8]> for Attribute {
    type Error = value::Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        Self::from_str(std::str::from_utf8(s).map_err(|err| color_err(s).with_err(err))?)
    }
}
