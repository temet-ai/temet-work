//! Draws the window offscreen and writes each shot as a PNG, with no window, Wayland or
//! compositor: GPUI's headless app context renders the frame through its wgpu renderer into
//! memory. Run it with `cargo run --example shots --features shots -- DIR [NAME...]`.
//!
//! Each shot is the Project page on the sample data in one theme at one window size; the sample
//! holds a selected row, a milestone with no issues, a group with no agents and long names.

use std::io::Write;
use std::sync::Arc;

use gpui::{AppContext, Font, FontWeight, HeadlessAppContext, font, px, size};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::pages::project::Project;
use temet_work_app::{sample, theme};

/// The shots: name, theme, width and height.
const SHOTS: &[(&str, &str, f32, f32)] = &[
    ("dark-1440", "dark", 1440., 900.),
    ("light-1440", "light", 1440., 900.),
    ("dark-1280", "dark", 1280., 800.),
    ("light-1280", "light", 1280., 800.),
    ("dark-1700", "dark", 1700., 1000.),
    ("light-1700", "light", 1700., 1000.),
];

fn main() -> gpui::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(dir) = args.first() else {
        anyhow_exit("usage: shots DIR [NAME...]");
    };
    let only = &args[1..];
    for &(name, theme_name, width, height) in SHOTS {
        if !only.is_empty() && !only.iter().any(|o| o == name) {
            continue;
        }
        let text = Arc::new(gpui_wgpu::CosmicTextSystem::new("Archivo"));
        let mut cx = HeadlessAppContext::with_platform(
            text,
            Arc::new(Assets),
            gpui_platform::current_headless_renderer,
        );
        let faces = cx.update(|cx| -> gpui::Result<String> {
            gpui_component::init(cx);
            cx.text_system().add_fonts(assets::faces())?;
            theme::apply(cx, theme_name).map_err(std::io::Error::other)?;
            // Each weight of Archivo resolves to a face of its own: 400, 600 and 700.
            let at = |weight: f32| {
                cx.text_system().resolve_font(&Font {
                    weight: FontWeight(weight),
                    ..font("Archivo")
                })
            };
            let weights = [at(400.), at(600.), at(700.)];
            let distinct = weights[0] != weights[1] && weights[1] != weights[2];
            Ok(format!(
                "text {}, code {}, Archivo 400/600/700 distinct faces: {distinct}",
                theme::text::family(cx),
                theme::text::code_family(cx)
            ))
        })?;
        let window = cx.open_window(size(px(width), px(height)), |window, cx| {
            let page = cx.new(|_| Project::new(sample::project_view(), sample::selected()));
            cx.new(|cx| Root::new(page, window, cx))
        })?;
        // At a scale of 1, so a shot's pixels match the design system's story shots.
        cx.simulate_window_scale_factor_change(window.into(), 1.);
        // Two frames: the issue tables measure their width in the first.
        for _ in 0..2 {
            cx.run_until_parked();
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))?;
        }
        cx.run_until_parked();
        let image = cx.capture_screenshot(window.into())?;
        let path = format!("{dir}/{name}.png");
        write_png(&path, image.width(), image.height(), image.as_raw())?;
        println!("{path}: {}x{}; {faces}", image.width(), image.height());
    }
    Ok(())
}

fn anyhow_exit(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2)
}

/// Writes 8-bit RGBA pixels as a PNG, with the image data in stored (uncompressed) deflate
/// blocks, so the tool needs no image library of its own.
fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    let row = width as usize * 4;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in rgba.chunks(row) {
        raw.push(0);
        raw.extend_from_slice(line);
    }
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(u8::from(i + 1 == blocks.len()));
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut out = std::fs::File::create(path)?;
    out.write_all(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])?;
    for (kind, data) in [(b"IHDR", &header), (b"IDAT", &zlib), (b"IEND", &Vec::new())] {
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        let mut chunk = kind.to_vec();
        chunk.extend_from_slice(data);
        out.write_all(&chunk)?;
        out.write_all(&crc32(&chunk).to_be_bytes())?;
    }
    Ok(())
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
