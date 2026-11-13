//! QR codes in photos (ADR-0073): the grey values of a photo are read with rqrr in a few
//! attempts, each at another size and smoothness, until one gives a code. Photos of a
//! screen carry its pixel grid and moiré, and a code may be small in the photo; no single
//! size reads them all. Pure Rust, so it is tested without a browser.

/// Longest edge, in pixels, of a photo as read: larger ones are averaged down
/// ([`grey_blocks`]).
pub const EDGE: usize = 3000;

/// The attempts, in order: longest edge in pixels and blur radius. Chosen with photos of
/// cards on paper and on screens, turned and cut (ADR-0073); the first reads most.
pub const ATTEMPTS: &[(usize, usize)] = &[(3000, 1), (2000, 1), (2400, 0), (1600, 1), (1200, 0)];

/// A grey image, one byte per pixel, row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grey {
    /// Grey values (0 black, 255 white).
    pub pixels: Vec<u8>,
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
}

/// How many pixels in each direction make one of the photo as read: the smallest whole
/// number that brings its longer edge to at most [`EDGE`].
pub fn block(width: usize, height: usize) -> usize {
    width.max(height).div_ceil(EDGE).max(1)
}

/// Grey values of RGBA rows as a canvas gives them (ITU-R BT.601 weights), `width` pixels
/// wide, averaged over blocks of `k` × `k` pixels: `width / k` values per `k` rows. Pixels
/// beyond the last whole block are left out. Averaging, unlike a browser that shrinks by
/// sampling, makes no moiré from a screen's pixel grid.
pub fn grey_blocks(rgba: &[u8], width: usize, k: usize) -> Vec<u8> {
    let (k, out_width) = (k.max(1), width / k.max(1));
    let rows = rgba.len() / 4 / width.max(1) / k;
    let mut out = Vec::with_capacity(out_width * rows);
    for row in 0..rows {
        for x in 0..out_width {
            let mut sum = 0u32;
            for dy in 0..k {
                let line = (row * k + dy) * width;
                for dx in 0..k {
                    let p = &rgba[(line + x * k + dx) * 4..][..4];
                    sum += 299 * u32::from(p[0]) + 587 * u32::from(p[1]) + 114 * u32::from(p[2]);
                }
            }
            out.push((sum / (1000 * (k * k) as u32)) as u8);
        }
    }
    out
}

impl Grey {
    fn at(&self, x: usize, y: usize) -> u8 {
        self.pixels[y * self.width + x]
    }

    /// Scaled so the longer edge is `edge`: averaged when smaller (no moiré from skipped
    /// pixels), interpolated when larger (small codes get enough pixels per module).
    fn fit(&self, edge: usize) -> Self {
        let (w, h) = (self.width, self.height);
        let long = w.max(h);
        let (nw, nh) = (
            (w * edge).div_ceil(long).max(1),
            (h * edge).div_ceil(long).max(1),
        );
        if long == edge {
            return self.clone();
        }
        let mut pixels = vec![0u8; nw * nh];
        if long > edge {
            for y in 0..nh {
                let (y0, y1) = (y * h / nh, ((y + 1) * h / nh).max(y * h / nh + 1));
                for x in 0..nw {
                    let (x0, x1) = (x * w / nw, ((x + 1) * w / nw).max(x * w / nw + 1));
                    let mut sum = 0u32;
                    for yy in y0..y1 {
                        for xx in x0..x1 {
                            sum += u32::from(self.at(xx, yy));
                        }
                    }
                    pixels[y * nw + x] = (sum / ((y1 - y0) * (x1 - x0)) as u32) as u8;
                }
            }
        } else {
            // Bilinear, in fixed point (1/256 of a pixel).
            for y in 0..nh {
                let sy = ((2 * y + 1) * h * 128 / nh).saturating_sub(128);
                let (y0, fy) = ((sy >> 8).min(h - 1), (sy & 255) as u32);
                let y1 = (y0 + 1).min(h - 1);
                for x in 0..nw {
                    let sx = ((2 * x + 1) * w * 128 / nw).saturating_sub(128);
                    let (x0, fx) = ((sx >> 8).min(w - 1), (sx & 255) as u32);
                    let x1 = (x0 + 1).min(w - 1);
                    let p = |xx, yy| u32::from(self.at(xx, yy));
                    let top = p(x0, y0) * (256 - fx) + p(x1, y0) * fx;
                    let bottom = p(x0, y1) * (256 - fx) + p(x1, y1) * fx;
                    pixels[y * nw + x] = ((top * (256 - fy) + bottom * fy) >> 16) as u8;
                }
            }
        }
        Self {
            pixels,
            width: nw,
            height: nh,
        }
    }

    /// Box blur of `radius`, twice in each direction (close to a Gaussian): smooths a
    /// screen's pixel grid away.
    fn blur(mut self, radius: usize) -> Self {
        if radius == 0 {
            return self;
        }
        let (w, h) = (self.width, self.height);
        let pass = |src: &[u8], horizontal: bool| -> Vec<u8> {
            let mut out = vec![0u8; src.len()];
            let (lines, len) = if horizontal { (h, w) } else { (w, h) };
            for i in 0..lines {
                let at = |j: usize| if horizontal { i * w + j } else { j * w + i };
                let (mut sum, mut count) = (0u32, 0u32);
                for j in 0..=radius.min(len - 1) {
                    sum += u32::from(src[at(j)]);
                    count += 1;
                }
                for j in 0..len {
                    out[at(j)] = (sum / count) as u8;
                    if j + radius + 1 < len {
                        sum += u32::from(src[at(j + radius + 1)]);
                        count += 1;
                    }
                    if j >= radius {
                        sum -= u32::from(src[at(j - radius)]);
                        count -= 1;
                    }
                }
            }
            out
        };
        for _ in 0..2 {
            self.pixels = pass(&pass(&self.pixels, true), false);
        }
        self
    }
}

/// The texts of the QR codes found in one attempt (an entry of [`ATTEMPTS`]).
pub fn attempt(grey: &Grey, (edge, radius): (usize, usize)) -> Vec<String> {
    let image = grey.fit(edge).blur(radius);
    let (w, pixels) = (image.width, &image.pixels);
    let mut prepared =
        rqrr::PreparedImage::prepare_from_greyscale(w, image.height, |x, y| pixels[y * w + x]);
    prepared
        .detect_grids()
        .iter()
        .filter_map(|g| g.decode().ok().map(|(_, text)| text))
        .collect()
}

/// The texts of the QR codes in a photo, from the first attempt that finds any (as the app
/// does, which lets the page answer between attempts).
#[cfg(test)]
fn read(grey: &Grey) -> Vec<String> {
    ATTEMPTS
        .iter()
        .map(|&a| attempt(grey, a))
        .find(|texts| !texts.is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = "Sommer;Jonas;14.03.1995;m;86150;Augsburg;deutsch;Bahnhofstraße 5;\
                        +491701234567;Augsburg-Stadt;BRK Bereitschaft Mitte;Stadtfest;\
                        SEG Behandlung;20.06.2027 - 08:30;;vegetarische Ernährung";

    /// A pseudo-random number in 0..n (a fixed sequence, so the test is repeatable).
    fn noise(state: &mut u64, n: u32) -> u32 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((*state >> 33) % u64::from(n)) as u32
    }

    /// A photo of a card shown on a screen: the code among lines of text, each screen pixel
    /// with a dark gap around it, photographed turned by `degrees`, with uneven light and
    /// sensor noise. `module` is screen pixels per module, `zoom` camera pixels per screen
    /// pixel.
    fn screen_photo(text: &str, degrees: f32, module: usize, zoom: f32) -> Grey {
        let code = qrcode::QrCode::new(text.as_bytes()).expect("fits a code");
        let n = code.width();
        let colors = code.to_colors();
        // The screen: the code with a quiet zone of 4 modules, text-like bars around it.
        let (sw, sh) = ((n + 8) * module * 2, (n + 8) * module * 3 / 2);
        let (qx, qy) = (sw - (n + 6) * module, (sh - n * module) / 2);
        let mut state = 7;
        let mut screen = vec![235u8; sw * sh];
        for line in 0..sh / (module * 4) {
            let y0 = line * module * 4 + module;
            let mut x = module;
            while x + module * 6 < qx - module * 4 {
                let len = module * (2 + noise(&mut state, 5) as usize);
                for y in y0..(y0 + module * 2).min(sh) {
                    for xx in x..(x + len).min(qx - module * 4) {
                        screen[y * sw + xx] = 40;
                    }
                }
                x += len + module;
            }
        }
        for my in 0..n {
            for mx in 0..n {
                if colors[my * n + mx] == qrcode::Color::Dark {
                    for y in 0..module {
                        for x in 0..module {
                            screen[(qy + my * module + y) * sw + qx + mx * module + x] = 25;
                        }
                    }
                }
            }
        }
        // The camera.
        let (cw, ch) = ((sw as f32 * zoom) as usize, (sh as f32 * zoom) as usize);
        let (sin, cos) = degrees.to_radians().sin_cos();
        let mut pixels = vec![0u8; cw * ch];
        for y in 0..ch {
            for x in 0..cw {
                let (dx, dy) = (x as f32 - cw as f32 / 2.0, y as f32 - ch as f32 / 2.0);
                let sx = (cos * dx + sin * dy) / zoom + sw as f32 / 2.0;
                let sy = (-sin * dx + cos * dy) / zoom + sh as f32 / 2.0;
                let value = if sx < 0.0 || sy < 0.0 || sx >= sw as f32 || sy >= sh as f32 {
                    15.0 // the screen's frame
                } else {
                    let grid = sx.fract() > 0.75 || sy.fract() > 0.8;
                    let v = f32::from(screen[sy as usize * sw + sx as usize]);
                    if grid { v * 0.45 } else { v }
                };
                let light = 0.75 + 0.25 * (x as f32 / cw as f32);
                let grain = noise(&mut state, 31) as f32 - 15.0;
                pixels[y * cw + x] = (value * light + grain).clamp(0.0, 255.0) as u8;
            }
        }
        Grey {
            pixels,
            width: cw,
            height: ch,
        }
    }

    // [utest->dsn~registration-card~2]
    #[test]
    fn a_card_photographed_on_a_screen_is_read() {
        for (degrees, module, zoom) in [(4.0, 4, 3.3), (-11.0, 3, 4.1)] {
            let photo = screen_photo(CARD, degrees, module, zoom);
            assert_eq!(
                read(&photo),
                [CARD],
                "turned {degrees}°, {module} screen pixels per module"
            );
        }
    }

    #[test]
    fn a_photo_without_a_code_gives_nothing() {
        let blank = Grey {
            pixels: vec![200; 640 * 480],
            width: 640,
            height: 480,
        };
        assert!(read(&blank).is_empty());
    }

    #[test]
    fn large_photos_are_averaged_in_blocks() {
        assert_eq!(block(3000, 2000), 1);
        assert_eq!(block(4032, 3024), 2);
        assert_eq!(block(5712, 4284), 2);
        assert_eq!(block(6001, 100), 3);
        // Two rows of five pixels: white, black, then grey ones; in blocks of 2 × 2, the
        // fifth column is left out.
        let px = |v: u8| [v, v, v, 255];
        let rows: Vec<u8> = [255, 0, 100, 100, 7, 255, 0, 100, 100, 7]
            .into_iter()
            .flat_map(px)
            .collect();
        assert_eq!(grey_blocks(&rows, 5, 2), [127, 100]);
        assert_eq!(grey_blocks(&rows, 5, 1).len(), 10, "1 × 1 is grey only");
        assert_eq!(grey_blocks(&[0, 0, 255, 255], 1, 1), [29], "blue is dark");
    }

    #[test]
    fn scaling_keeps_the_picture() {
        let grey = Grey {
            pixels: (0..16).map(|i| (i * 16) as u8).collect(),
            width: 4,
            height: 4,
        };
        let down = grey.fit(2);
        assert_eq!((down.width, down.height), (2, 2));
        assert_eq!(down.pixels, [40, 72, 168, 200], "averages of 2×2");
        let up = grey.fit(8);
        assert_eq!((up.width, up.height), (8, 8));
        assert_eq!(up.pixels[0], 0);
        assert_eq!(up.pixels[63], 240);
        let flat = Grey {
            pixels: vec![99; 25],
            width: 5,
            height: 5,
        };
        assert!(flat.blur(2).pixels.iter().all(|&p| p == 99));
    }
}
