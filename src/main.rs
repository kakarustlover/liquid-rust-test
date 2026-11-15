//! Liquid Rust: native Android demo (NativeActivity + winit + wgpu 30 + liquid-rust).
//! Backgrounds are the JPEGs in ../assets (bg0.jpg .. bg2.jpg).

use std::sync::Arc;
use std::time::Instant;

use liquid_rust::{Color, ContainerId, Frame, Glass, GlassId, Material, Rect, Renderer, Scene, Spring, Vec2};
use winit::application::ApplicationHandler;
use winit::event::{TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_os = "android")]
use winit::platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid};

// ───────────────────────── constants ─────────────────────────

const BG_COUNT: usize = 3;
const BG_NAMES: [&str; BG_COUNT] = ["MEADOW", "VALLEY", "CANYON"];
const PRESETS: [&str; 4] = ["REGULAR", "CLEAR", "TINTED", "BAR"];
const TINTS: [Color; 4] = [Color::RED, Color::YELLOW, Color::GREEN, Color::BLUE];
const SL_NAMES: [&str; 7] = ["FROST", "REFRACTION", "DEPTH", "DISPERSION", "SPECULAR", "TRANSPARENCY", "MERGE"];
const THUMB_W: f32 = 44.0;

fn bg_bytes(i: usize) -> &'static [u8] {
    match i % BG_COUNT {
        0 => &include_bytes!("../assets/bg0.jpg")[..],
        1 => &include_bytes!("../assets/bg1.jpg")[..],
        _ => &include_bytes!("../assets/bg2.jpg")[..],
    }
}

fn fmt_val(i: usize, v: f32) -> String {
    match i {
        0 => format!("{:.0} PT", v * 30.0),
        2 => format!("{:.0} PT", v * 40.0),
        6 => format!("{:.0} PT", v * 80.0),
        _ => format!("{:.2}", v),
    }
}

fn toggle_mat(on: bool) -> Material {
    if on {
        Material::tinted(Color::BLUE.with_alpha(0.85))
    } else {
        Material::regular()
    }
}

// ───────────────────────── tiny 5x7 bitmap font ─────────────────────────

fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '.' => [0, 0, 0, 0, 0, 0b01100, 0b01100],
        ':' => [0, 0b01100, 0b01100, 0, 0b01100, 0b01100, 0],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '%' => [0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011],
        '<' => [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
        '>' => [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000],
        '*' => [0, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0],
        _ => [0; 7],
    }
}

#[derive(Clone, Copy)]
struct IRect {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

impl IRect {
    fn intersects(&self, o: &IRect) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }
    fn clamped(self, w: i32, h: i32) -> IRect {
        IRect { x0: self.x0.max(0), y0: self.y0.max(0), x1: self.x1.min(w), y1: self.y1.min(h) }
    }
    fn empty(&self) -> bool {
        self.x0 >= self.x1 || self.y0 >= self.y1
    }
}

struct Label {
    x: i32,
    y: i32,
    fp: i32,
    text: String,
    right: bool,
}

impl Label {
    fn width(&self) -> i32 {
        let n = self.text.chars().count() as i32;
        if n == 0 {
            0
        } else {
            (n * 6 - 1) * self.fp
        }
    }
    fn left(&self) -> i32 {
        if self.right {
            self.x - self.width()
        } else {
            self.x
        }
    }
    fn bbox(&self) -> IRect {
        let l = self.left();
        IRect { x0: l - 2, y0: self.y - 2, x1: l + self.width() + self.fp + 3, y1: self.y + 8 * self.fp + 3 }
    }
}

fn put_rect(buf: &mut [u8], bw: i32, clip: IRect, x: i32, y: i32, w: i32, h: i32, col: [u8; 3], a: u32) {
    let x0 = x.max(clip.x0);
    let y0 = y.max(clip.y0);
    let x1 = (x + w).min(clip.x1);
    let y1 = (y + h).min(clip.y1);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    for yy in y0..y1 {
        let row = (yy * bw) as usize;
        for xx in x0..x1 {
            let o = (row + xx as usize) * 4;
            for c in 0..3 {
                let d = buf[o + c] as u32;
                buf[o + c] = ((d * (255 - a) + col[c] as u32 * a) / 255) as u8;
            }
        }
    }
}

fn draw_text(buf: &mut [u8], bw: i32, clip: IRect, x: i32, y: i32, fp: i32, text: &str) {
    for pass in 0..2 {
        let (ox, oy, col, a) = if pass == 0 { (fp / 2 + 1, fp / 2 + 1, [0u8, 0, 0], 150u32) } else { (0, 0, [255u8, 255, 255], 255u32) };
        for (i, ch) in text.chars().enumerate() {
            let g = glyph(ch);
            for ry in 0..7usize {
                for cx in 0..5i32 {
                    if g[ry] & (1 << (4 - cx)) != 0 {
                        put_rect(buf, bw, clip, x + ox + (i as i32 * 6 + cx) * fp, y + oy + ry as i32 * fp, fp, fp, col, a);
                    }
                }
            }
        }
    }
}

// ───────────────────────── persistent UI state ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Act {
    Prev,
    Next,
    Light,
    Plus,
    Dark,
    Contrast,
    Frosty,
    Calm,
    Physics,
    Preset,
    Tint(usize),
}

#[derive(Clone)]
struct Ui {
    bg: usize,
    preset: usize,
    tint: usize,
    dark: bool,
    contrast: bool,
    frosty: bool,
    calm: bool,
    physics: bool,
    light_auto: bool,
    sl: [f32; 7],
}

impl Ui {
    fn new() -> Ui {
        let mut u = Ui {
            bg: 0,
            preset: 1,
            tint: 3,
            dark: false,
            contrast: false,
            frosty: false,
            calm: false,
            physics: true,
            light_auto: false,
            sl: [0.0; 7],
        };
        u.sync_from_preset();
        u.sl[5] = 0.5;
        u.sl[6] = 28.0 / 80.0;
        u
    }
    fn preset_mat(&self) -> Material {
        match self.preset {
            0 => Material::regular(),
            1 => Material::clear(),
            2 => Material::tinted(TINTS[self.tint].with_alpha(0.6)),
            _ => Material::clear_bar(),
        }
    }
    fn sync_from_preset(&mut self) {
        let m = self.preset_mat();
        self.sl[0] = (m.frost / 30.0).clamp(0.0, 1.0);
        self.sl[1] = m.refraction.clamp(0.0, 1.0);
        self.sl[2] = (m.depth / 40.0).clamp(0.0, 1.0);
        self.sl[3] = m.dispersion.clamp(0.0, 1.0);
        self.sl[4] = m.specular.clamp(0.0, 1.0);
    }
    fn show_mat(&self) -> Material {
        let mut m = self.preset_mat();
        m.frost = self.sl[0] * 30.0;
        m.refraction = self.sl[1];
        m.depth = self.sl[2] * 40.0;
        m.dispersion = self.sl[3];
        m.specular = self.sl[4];
        m
    }
}

struct Press {
    id: Option<GlassId>,
    start: Vec2,
    moved: bool,
    slider: Option<usize>,
    tid: u64,
}

// ───────────────────────── GPU + scene ─────────────────────────

struct Gfx {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_fmt: wgpu::TextureFormat,
    renderer: Renderer,
    scene: Scene,
    content_tex: wgpu::Texture,
    content_view: wgpu::TextureView,
    base: Vec<u8>,
    work: Vec<u8>,
    pw: u32,
    ph: u32,
    scale: f32,
    w: f32,
    labels: Vec<Label>,
    bg_lbl: usize,
    preset_lbl: usize,
    val_lbl: Vec<usize>,
    val_area: Vec<IRect>,
    buttons: Vec<(GlassId, Act)>,
    thumbs: Vec<GlassId>,
    row_cy: [f32; 7],
    tx0: f32,
    span: f32,
    showcase: Vec<GlassId>,
    liquid: ContainerId,
    menu: ContainerId,
    plus: GlassId,
    menu_items: Vec<GlassId>,
    content_dirty: bool,
    t_light: f32,
    last: Instant,
    press: Option<Press>,
    ui: Ui,
}

impl Gfx {
    fn new(window: Arc<Window>, ui: Ui) -> Gfx {
        let size = window.inner_size();
        let pw = size.width.max(1);
        let ph = size.height.max(1);
        let scale = window.scale_factor() as f32;
        let w = pw as f32 / scale;
        let h = ph as f32 / scale;

        // wgpu setup
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance.create_surface(window.clone()).expect("create_surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("request_adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("liquid"),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");

        let caps = surface.get_capabilities(&adapter);
        let fmt = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        let render_fmt = fmt.add_srgb_suffix();
        let mut config = surface.get_default_config(&adapter, pw, ph).expect("surface config");
        config.format = fmt;
        config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        config.present_mode = wgpu::PresentMode::Fifo;
        config.view_formats = if render_fmt != fmt { vec![render_fmt] } else { vec![] };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, render_fmt);
        let content_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("content"),
            size: wgpu::Extent3d { width: pw, height: ph, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let content_view = content_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // scene
        let mut scene = Scene::new();
        scene.appearance.dark = ui.dark;
        scene.appearance.increase_contrast = ui.contrast;
        scene.appearance.reduce_transparency = ui.frosty;
        scene.appearance.reduce_motion = ui.calm;
        scene.appearance.transparency = ui.sl[5];
        scene.physics = ui.physics;

        let mut buttons: Vec<(GlassId, Act)> = Vec::new();
        let bs = 52.0f32;
        let by = 14.0f32;

        let bar = scene.add_container(14.0, 0);
        for (i, act) in [Act::Prev, Act::Next, Act::Light].into_iter().enumerate() {
            let x = 20.0 + i as f32 * (bs + 16.0);
            let mat = if act == Act::Light { toggle_mat(ui.light_auto) } else { Material::regular() };
            let id = scene.add(Glass::new(Rect::new(x, by, bs, bs)).material(mat).interactive().container(bar));
            buttons.push((id, act));
        }
        let menu = scene.add_container(16.0, 1);
        let plus = scene.add(Glass::new(Rect::new(w - 20.0 - bs, by, bs, bs)).material(Material::regular()).interactive().container(menu));
        buttons.push((plus, Act::Plus));

        // liquid showcase (lens + blobs in one container)
        let liquid = scene.add_container(ui.sl[6] * 80.0, 0);
        let m = ui.show_mat();
        let mut showcase = Vec::new();
        let specs = [
            (0.12 * w, 160.0f32, 140.0f32, false, false),
            (0.52 * w, 206.0, 96.0, false, false),
            (0.18 * w, 340.0, 80.0, true, false),
            (0.56 * w, 345.0, 108.0, false, true),
        ];
        for (x, y, s, inter, lens) in specs {
            let mut g = Glass::new(Rect::new(x, y, s, s)).material(m).container(liquid);
            if inter {
                g = g.interactive();
            }
            if lens {
                g = g.lens();
            }
            showcase.push(scene.add(g));
        }

        // sliders
        let dock_h = 64.0f32;
        let dock_y = h - 28.0 - dock_h;
        let pitch = 36.0f32;
        let first = dock_y - 30.0 - 7.0 * pitch;
        let tx0 = 134.0f32;
        let tx1 = w - 80.0;
        let span = (tx1 - tx0) - THUMB_W;
        let mut thumbs = Vec::new();
        let mut row_cy = [0.0f32; 7];
        for i in 0..7 {
            let cy = first + i as f32 * pitch + pitch / 2.0;
            row_cy[i] = cy;
            scene.add(Glass::new(Rect::new(tx0, cy - 6.0, tx1 - tx0, 12.0)).material(Material::clear()).z(1));
            let t = scene.add(Glass::new(Rect::new(tx0 + ui.sl[i] * span, cy - 15.0, THUMB_W, 30.0)).material(Material::regular()).interactive().z(3));
            thumbs.push(t);
        }

        // dock
        let dock_x = 16.0f32;
        let dock_w = w - 32.0;
        scene.add(Glass::new(Rect::new(dock_x, dock_y, dock_w, dock_h)).material(Material::clear_bar()).z(1));
        let acts = [Act::Dark, Act::Contrast, Act::Frosty, Act::Calm, Act::Physics, Act::Preset];
        let bsz = 46.0f32;
        let gap = (dock_w - 6.0 * bsz) / 7.0;
        for (i, a) in acts.iter().enumerate() {
            let x = dock_x + gap + i as f32 * (bsz + gap);
            let on = match a {
                Act::Dark => ui.dark,
                Act::Contrast => ui.contrast,
                Act::Frosty => ui.frosty,
                Act::Calm => ui.calm,
                Act::Physics => ui.physics,
                _ => false,
            };
            let id = scene.add(Glass::new(Rect::new(x, dock_y + 9.0, bsz, bsz)).material(toggle_mat(on)).interactive().z(3));
            buttons.push((id, *a));
        }

        // labels (baked into the content layer, never under big glass)
        let px = |v: f32| (v * scale).round() as i32;
        let mut labels: Vec<Label> = Vec::new();
        let fp_t = ((scale * 2.8).round() as i32).max(3);
        labels.push(Label { x: px(24.0), y: px(80.0), fp: fp_t, text: "LIQUID RUST".to_string(), right: false });
        let fp_m = ((scale * 1.45).round() as i32).max(2);
        let bg_lbl = labels.len();
        labels.push(Label { x: px(24.0), y: px(112.0), fp: fp_m, text: String::new(), right: false });
        let preset_lbl = labels.len();
        labels.push(Label { x: px(24.0), y: px(132.0), fp: fp_m, text: String::new(), right: false });

        let fp_i = ((scale * 2.0).round() as i32).max(3);
        let icons = ["<", ">", "*"];
        for (i, t) in icons.iter().enumerate() {
            let cx = 20.0 + i as f32 * (bs + 16.0) + bs / 2.0;
            labels.push(Label { x: px(cx) - (5 * fp_i) / 2, y: px(by + bs / 2.0) - (7 * fp_i) / 2, fp: fp_i, text: t.to_string(), right: false });
        }
        labels.push(Label { x: px(w - 20.0 - bs / 2.0) - (5 * fp_i) / 2, y: px(by + bs / 2.0) - (7 * fp_i) / 2, fp: fp_i, text: "+".to_string(), right: false });

        let fp_l = ((scale * 1.4).round() as i32).max(2);
        let mut val_lbl = Vec::new();
        let mut val_area = Vec::new();
        for i in 0..7 {
            let cy = row_cy[i];
            labels.push(Label { x: px(24.0), y: px(cy) - (7 * fp_l) / 2, fp: fp_l, text: SL_NAMES[i].to_string(), right: false });
            val_lbl.push(labels.len());
            labels.push(Label { x: px(w - 24.0), y: px(cy) - (7 * fp_l) / 2, fp: fp_l, text: fmt_val(i, ui.sl[i]), right: true });
            val_area.push(IRect { x0: px(w - 24.0 - 66.0), y0: px(cy - 12.0), x1: px(w - 24.0) + 6, y1: px(cy + 12.0) });
        }

        let fp_d = ((scale * 1.05).round() as i32).max(2);
        let dnames = ["DARK", "CONTRAST", "FROSTY", "CALM", "PHYSICS", "GLASS"];
        for (i, n) in dnames.iter().enumerate() {
            let cx = dock_x + gap + i as f32 * (bsz + gap) + bsz / 2.0;
            let wpx = (n.len() as i32 * 6 - 1) * fp_d;
            labels.push(Label { x: px(cx) - wpx / 2, y: px(dock_y - 14.0), fp: fp_d, text: n.to_string(), right: false });
        }

        let mut g = Gfx {
            window,
            surface,
            device,
            queue,
            config,
            render_fmt,
            renderer,
            scene,
            content_tex,
            content_view,
            base: Vec::new(),
            work: vec![0u8; (pw as usize) * (ph as usize) * 4],
            pw,
            ph,
            scale,
            w,
            labels,
            bg_lbl,
            preset_lbl,
            val_lbl,
            val_area,
            buttons,
            thumbs,
            row_cy,
            tx0,
            span,
            showcase,
            liquid,
            menu,
            plus,
            menu_items: Vec::new(),
            content_dirty: true,
            t_light: 0.0,
            last: Instant::now(),
            press: None,
            ui,
        };
        g.update_preset_label();
        g.load_bg();
        g.refresh_full();
        g
    }

    // ── content layer ──

    fn load_bg(&mut self) {
        let img = image::load_from_memory(bg_bytes(self.ui.bg))
            .expect("decode background")
            .resize_to_fill(self.pw, self.ph, image::imageops::FilterType::Triangle);
        self.base = img.to_rgba8().into_raw();
        let name = BG_NAMES[self.ui.bg % BG_COUNT];
        self.labels[self.bg_lbl].text = format!("{}/{}  {}", self.ui.bg % BG_COUNT + 1, BG_COUNT, name);
    }

    fn update_preset_label(&mut self) {
        self.labels[self.preset_lbl].text = format!("GLASS: {}  -  DRAG THE LENS", PRESETS[self.ui.preset]);
    }

    fn refresh_full(&mut self) {
        let r = IRect { x0: 0, y0: 0, x1: self.pw as i32, y1: self.ph as i32 };
        self.refresh(r);
    }

    fn refresh_label_row(&mut self, idx: usize) {
        let r = {
            let l = &self.labels[idx];
            IRect { x0: 0, y0: l.y - 4, x1: self.pw as i32, y1: l.y + 8 * l.fp + 6 }
        };
        self.refresh(r);
    }

    fn refresh(&mut self, r: IRect) {
        let pw = self.pw as i32;
        let r = r.clamped(pw, self.ph as i32);
        if r.empty() || self.base.is_empty() {
            return;
        }
        let rw = (r.x1 - r.x0) as usize;
        let rh = (r.y1 - r.y0) as usize;
        for y in r.y0..r.y1 {
            let o = (y as usize * pw as usize + r.x0 as usize) * 4;
            self.work[o..o + rw * 4].copy_from_slice(&self.base[o..o + rw * 4]);
        }
        for l in &self.labels {
            if l.bbox().intersects(&r) {
                draw_text(&mut self.work, pw, r, l.left(), l.y, l.fp, &l.text);
            }
        }
        let mut packed = Vec::with_capacity(rw * rh * 4);
        for y in r.y0..r.y1 {
            let o = (y as usize * pw as usize + r.x0 as usize) * 4;
            packed.extend_from_slice(&self.work[o..o + rw * 4]);
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.content_tex,
                mip_level: 0,
                origin: wgpu::Origin3d { x: r.x0 as u32, y: r.y0 as u32, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &packed,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(rw as u32 * 4), rows_per_image: Some(rh as u32) },
            wgpu::Extent3d { width: rw as u32, height: rh as u32, depth_or_array_layers: 1 },
        );
        self.content_dirty = true;
    }

    // ── interaction ──

    fn btn(&self, a: Act) -> Option<GlassId> {
        self.buttons.iter().find(|(_, x)| *x == a).map(|(id, _)| *id)
    }

    fn set_toggle(&mut self, a: Act, on: bool) {
        if let Some(id) = self.btn(a) {
            self.scene.set_material(id, toggle_mat(on));
        }
    }

    fn apply_showcase(&mut self) {
        let m = self.ui.show_mat();
        for &id in &self.showcase {
            self.scene.set_material(id, m);
        }
    }

    fn place_thumb(&mut self, i: usize) {
        let v = self.ui.sl[i];
        let r = Rect::new(self.tx0 + v * self.span, self.row_cy[i] - 15.0, THUMB_W, 30.0);
        self.scene.set_frame(self.thumbs[i], r);
    }

    fn update_value(&mut self, i: usize) {
        let idx = self.val_lbl[i];
        self.labels[idx].text = fmt_val(i, self.ui.sl[i]);
        let area = self.val_area[i];
        self.refresh(area);
    }

    fn drag_slider(&mut self, i: usize, x_pt: f32) {
        let v = ((x_pt - THUMB_W / 2.0 - self.tx0) / self.span).clamp(0.0, 1.0);
        self.ui.sl[i] = v;
        self.place_thumb(i);
        self.update_value(i);
        if i < 5 {
            self.apply_showcase();
        } else if i == 5 {
            self.scene.appearance.transparency = v;
        } else {
            self.scene.set_container_spacing(self.liquid, v * 80.0);
        }
    }

    fn set_bg(&mut self, i: usize) {
        self.ui.bg = i % BG_COUNT;
        self.load_bg();
        self.refresh_full();
    }

    fn set_preset(&mut self, p: usize) {
        self.ui.preset = p % 4;
        self.ui.sync_from_preset();
        self.apply_showcase();
        for i in 0..5 {
            self.place_thumb(i);
            self.update_value(i);
        }
        self.update_preset_label();
        let idx = self.preset_lbl;
        self.refresh_label_row(idx);
    }

    fn open_menu(&mut self) {
        let plus = self.plus;
        let cx = self.w - 20.0 - 26.0;
        for (i, c) in TINTS.iter().enumerate() {
            let y = 14.0 + 52.0 + 18.0 + i as f32 * 64.0;
            let glass = Glass::new(Rect::new(cx - 22.0, y, 44.0, 44.0))
                .material(Material::tinted(c.with_alpha(0.85)))
                .interactive()
                .container(self.menu);
            let id = self.scene.expand_from(plus, glass, Spring::new(0.55, 0.25));
            self.menu_items.push(id);
            self.buttons.push((id, Act::Tint(i)));
        }
    }

    fn close_menu(&mut self) {
        let plus = self.plus;
        let items = std::mem::take(&mut self.menu_items);
        for id in items {
            self.scene.collapse_into(id, plus, Spring::new(0.35, 0.0));
            self.buttons.retain(|(b, _)| *b != id);
        }
    }

    fn act(&mut self, a: Act) {
        match a {
            Act::Prev => self.set_bg(self.ui.bg + BG_COUNT - 1),
            Act::Next => self.set_bg(self.ui.bg + 1),
            Act::Light => {
                self.ui.light_auto = !self.ui.light_auto;
                let on = self.ui.light_auto;
                self.set_toggle(a, on);
            }
            Act::Plus => {
                if self.menu_items.is_empty() {
                    self.open_menu();
                } else {
                    self.close_menu();
                }
            }
            Act::Dark => {
                self.ui.dark = !self.ui.dark;
                self.scene.appearance.dark = self.ui.dark;
                let on = self.ui.dark;
                self.set_toggle(a, on);
            }
            Act::Contrast => {
                self.ui.contrast = !self.ui.contrast;
                self.scene.appearance.increase_contrast = self.ui.contrast;
                let on = self.ui.contrast;
                self.set_toggle(a, on);
            }
            Act::Frosty => {
                self.ui.frosty = !self.ui.frosty;
                self.scene.appearance.reduce_transparency = self.ui.frosty;
                let on = self.ui.frosty;
                self.set_toggle(a, on);
            }
            Act::Calm => {
                self.ui.calm = !self.ui.calm;
                self.scene.appearance.reduce_motion = self.ui.calm;
                let on = self.ui.calm;
                self.set_toggle(a, on);
            }
            Act::Physics => {
                self.ui.physics = !self.ui.physics;
                self.scene.physics = self.ui.physics;
                let on = self.ui.physics;
                self.set_toggle(a, on);
            }
            Act::Preset => self.set_preset(self.ui.preset + 1),
            Act::Tint(i) => {
                self.ui.tint = i;
                self.set_preset(2);
                self.close_menu();
            }
        }
    }

    fn on_touch(&mut self, phase: TouchPhase, x: f64, y: f64, tid: u64) {
        let p = Vec2::new(x as f32 / self.scale, y as f32 / self.scale);
        match phase {
            TouchPhase::Started => {
                if self.press.is_some() {
                    return;
                }
                let id = self.scene.pointer_down(p);
                let mut slider = None;
                if let Some(g) = id {
                    for i in 0..self.thumbs.len() {
                        if self.thumbs[i] == g {
                            slider = Some(i);
                        }
                    }
                }
                self.press = Some(Press { id, start: p, moved: false, slider, tid });
                if let Some(i) = slider {
                    self.drag_slider(i, p.x);
                }
            }
            TouchPhase::Moved => {
                let mut slider = None;
                if let Some(pr) = self.press.as_mut() {
                    if pr.tid != tid {
                        return;
                    }
                    if (p.x - pr.start.x).hypot(p.y - pr.start.y) > 10.0 {
                        pr.moved = true;
                    }
                    slider = pr.slider;
                } else {
                    return;
                }
                self.scene.pointer_move(p);
                if let Some(i) = slider {
                    self.drag_slider(i, p.x);
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let pr = match self.press.take() {
                    Some(pr) => pr,
                    None => return,
                };
                if pr.tid != tid {
                    self.press = Some(pr);
                    return;
                }
                self.scene.pointer_up();
                if phase == TouchPhase::Ended && !pr.moved && pr.slider.is_none() {
                    if let Some(g) = pr.id {
                        let act = self.buttons.iter().find(|(b, _)| *b == g).map(|(_, a)| *a);
                        if let Some(a) = act {
                            self.act(a);
                        }
                    }
                }
            }
        }
    }

    // ── frame ──

    /// Returns true while another frame is needed.
    fn render(&mut self) -> bool {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.05);
        self.last = now;
        if self.ui.light_auto {
            self.t_light += dt;
            self.scene.set_light_angle((self.t_light * 40.0) % 360.0);
        }
        let animating = self.scene.update(dt);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return true;
            }
            _ => return true,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(self.render_fmt), ..Default::default() });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.renderer.render(
            &self.device,
            &self.queue,
            &mut enc,
            &Frame {
                content: &self.content_view,
                target: &view,
                size: (self.pw, self.ph),
                scale: self.scale,
                content_changed: self.content_dirty,
            },
            &self.scene,
        );
        self.content_dirty = false;
        self.queue.submit(std::iter::once(enc.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);
        animating || self.ui.light_auto
    }
}

// ───────────────────────── app / event loop ─────────────────────────

struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    ui: Ui,
}

impl App {
    fn make_gfx(&mut self) {
        if self.gfx.is_some() {
            return;
        }
        if let Some(w) = &self.window {
            let s = w.inner_size();
            if s.width > 0 && s.height > 0 {
                self.gfx = Some(Gfx::new(w.clone(), self.ui.clone()));
            }
        }
    }

    fn drop_gfx(&mut self) {
        if let Some(g) = self.gfx.take() {
            self.ui = g.ui;
        }
    }

    fn redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            let w = el.create_window(Window::default_attributes().with_title("Liquid Rust")).expect("create_window");
            self.window = Some(Arc::new(w));
        }
        self.make_gfx();
        self.redraw();
    }

    fn suspended(&mut self, _el: &ActiveEventLoop) {
        self.drop_gfx();
        self.window = None;
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(sz) => {
                let changed = match &self.gfx {
                    Some(g) => g.pw != sz.width || g.ph != sz.height,
                    None => false,
                };
                if changed {
                    self.drop_gfx();
                }
                self.make_gfx();
                self.redraw();
            }
            WindowEvent::Touch(t) => {
                if let Some(g) = self.gfx.as_mut() {
                    g.on_touch(t.phase, t.location.x, t.location.y, t.id);
                }
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                let keep = match self.gfx.as_mut() {
                    Some(g) => g.render(),
                    None => false,
                };
                if keep {
                    self.redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: AndroidApp) {
    let event_loop = EventLoop::builder().with_android_app(app).build().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut a = App { window: None, gfx: None, ui: Ui::new() };
    event_loop.run_app(&mut a).expect("run_app");
}
