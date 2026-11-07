//! sRGB colours and WCAG 2 contrast.

/// A colour in sRGB with components from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Srgb {
    /// Red, 0–1.
    pub r: f64,
    /// Green, 0–1.
    pub g: f64,
    /// Blue, 0–1.
    pub b: f64,
    /// Opacity, 0–1.
    pub alpha: f64,
}

impl Srgb {
    /// An opaque colour.
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            r,
            g,
            b,
            alpha: 1.0,
        }
    }

    /// Parses `#rrggbb` or `#rrggbbaa` (case-insensitive).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let digits = hex.strip_prefix('#')?;
        if !(digits.len() == 6 || digits.len() == 8) || !digits.is_ascii() {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
        let channel = |i: usize| byte(i).map(|b| f64::from(b) / 255.0);
        Some(Self {
            r: channel(0)?,
            g: channel(2)?,
            b: channel(4)?,
            alpha: if digits.len() == 8 { channel(6)? } else { 1.0 },
        })
    }

    /// The colour as 8-bit channels (alpha ignored).
    pub fn to_bytes(self) -> [u8; 3] {
        let byte = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        [byte(self.r), byte(self.g), byte(self.b)]
    }

    /// `#rrggbb` in lower case (alpha ignored).
    pub fn to_hex(self) -> String {
        let [r, g, b] = self.to_bytes();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// CSS notation: hex if opaque, otherwise `rgb(r g b / alpha)`.
    pub fn to_css(self) -> String {
        if self.alpha >= 1.0 {
            self.to_hex()
        } else {
            let [r, g, b] = self.to_bytes();
            format!("rgb({r} {g} {b} / {})", trim(self.alpha))
        }
    }

    /// WCAG 2 relative luminance.
    pub fn relative_luminance(self) -> f64 {
        fn linear(c: f64) -> f64 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }

    /// This colour painted over an opaque `background`.
    pub fn over(self, background: Srgb) -> Srgb {
        let mix = |fg: f64, bg: f64| fg * self.alpha + bg * (1.0 - self.alpha);
        Srgb::rgb(
            mix(self.r, background.r),
            mix(self.g, background.g),
            mix(self.b, background.b),
        )
    }
}

/// WCAG 2 contrast ratio between two opaque colours, from 1 to 21.
pub fn contrast_ratio(a: Srgb, b: Srgb) -> f64 {
    let (la, lb) = (a.relative_luminance(), b.relative_luminance());
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Formats a ratio for people, rounded **down** to two decimals: WCAG thresholds must
/// not be met by rounding up (4.499:1 is not 4.5:1).
pub fn format_ratio(ratio: f64) -> String {
    format!("{:.2}:1", (ratio * 100.0).floor() / 100.0)
}

pub(crate) fn trim(value: f64) -> String {
    let text = format!("{value:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(h: &str) -> Srgb {
        Srgb::from_hex(h).unwrap()
    }

    #[test]
    fn black_and_white() {
        assert!((contrast_ratio(hex("#000000"), hex("#ffffff")) - 21.0).abs() < 1e-9);
        assert!((contrast_ratio(hex("#ffffff"), hex("#ffffff")) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn known_grey_threshold() {
        // The well-known lightest grey that passes 4.5:1 on white is #767676.
        assert!(contrast_ratio(hex("#767676"), hex("#ffffff")) >= 4.5);
        assert!(contrast_ratio(hex("#777777"), hex("#ffffff")) < 4.5);
    }

    #[test]
    fn ratio_is_symmetric() {
        let (a, b) = (hex("#1d5cc0"), hex("#f5f6f8"));
        assert_eq!(contrast_ratio(a, b), contrast_ratio(b, a));
    }

    #[test]
    fn formatting_never_rounds_up() {
        assert_eq!(format_ratio(4.4999), "4.49:1");
        assert_eq!(format_ratio(21.0), "21.00:1");
    }

    #[test]
    fn hex_parsing_and_output() {
        assert_eq!(hex("#1D5CC0").to_hex(), "#1d5cc0");
        assert_eq!(hex("#00000080").to_css(), "rgb(0 0 0 / 0.502)");
        assert!(Srgb::from_hex("1d5cc0").is_none());
        assert!(Srgb::from_hex("#12345").is_none());
        assert!(Srgb::from_hex("#gg0000").is_none());
    }

    #[test]
    fn transparent_colour_is_composited() {
        let half_black = Srgb {
            alpha: 0.5,
            ..hex("#000000")
        };
        let mixed = half_black.over(hex("#ffffff"));
        assert_eq!(mixed.to_hex(), "#808080");
    }
}
