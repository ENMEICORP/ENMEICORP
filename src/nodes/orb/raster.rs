//! THE BLACK FOREST // NODE 03: RADIANT SOFTWARE RASTERIZER
//!
//! Pure-CPU raytracing engine:
//! - Stretched night forest background (BGrad.png)
//! - Crystalline Fresnel limb-darkened sphere
//! - High-saturation Solar Gold star beacons
//! - CSS Hairline dock: [ ? ] Help, [ ~ ] Shuffle, and [ ⑉ ] Audio Noise
//! - Cinematic 750ms CRT shutdown sequence

use super::math::{Arcball, Quat, Vec3};
use super::topology::{CelestialLattice, RING_SAMPLES};
use std::path::Path;

const CAMERA_DIST: f32 = 4.2;

const COLOR_FRESNEL_R: f32 = 90.0;
const COLOR_FRESNEL_G: f32 = 160.0;
const COLOR_FRESNEL_B: f32 = 240.0;

const COLOR_LATTICE_R: f32 = 85.0;
const COLOR_LATTICE_G: f32 = 105.0;
const COLOR_LATTICE_B: f32 = 125.0;

// =========================================================================
// MONOGRAM DISPLAY (96x16 PIXEL-PERFECT)
// =========================================================================

pub struct EnmeiLogoImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl EnmeiLogoImage {
    pub fn load() -> Self {
        const LOGO_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/enmeilogo.png"));
        if let Ok(img) = image::load_from_memory_with_format(LOGO_BYTES, image::ImageFormat::Png) {
            let rgb = img.to_rgba8();
            let (w, h) = rgb.dimensions();
            let mut pixels = Vec::with_capacity((w * h) as usize);
            for p in rgb.pixels() {
                let r = p[0] as u32;
                let g = p[1] as u32;
                let b = p[2] as u32;
                let a = p[3] as u32;
                pixels.push((a << 24) | (r << 16) | (g << 8) | b);
            }
            Self { width: w as usize, height: h as usize, pixels }
        } else {
            Self { width: 0, height: 0, pixels: Vec::new() }
        }
    }

    pub fn render_pixel_perfect(&self, buffer: &mut [u32], buf_w: usize, buf_h: usize, dst_x: usize, dst_y: usize, opacity: f32) {
        if self.width == 0 || opacity <= 0.05 { return; }

        for py in 0..self.height {
            let dy = dst_y + py;
            if dy >= buf_h { break; }
            let row_offset = dy * buf_w;

            for px in 0..self.width {
                let dx = dst_x + px;
                if dx >= buf_w { break; }

                let p = self.pixels[py * self.width + px];
                let a = ((p >> 24) & 0xFF) as f32 / 255.0 * opacity;
                if a <= 0.02 { continue; }

                let r = ((p >> 16) & 0xFF) as f32;
                let g = ((p >> 8) & 0xFF) as f32;
                let b = (p & 0xFF) as f32;

                let idx = row_offset + dx;
                let curr = buffer[idx];
                let cr = ((curr >> 16) & 0xFF) as f32;
                let cg = ((curr >> 8) & 0xFF) as f32;
                let cb = (curr & 0xFF) as f32;

                let nr = (r * a + cr * (1.0 - a)).min(255.0) as u32;
                let ng = (g * a + cg * (1.0 - a)).min(255.0) as u32;
                let nb = (b * a + cb * (1.0 - a)).min(255.0) as u32;

                buffer[idx] = (nr << 16) | (ng << 8) | nb;
            }
        }
    }
}

// =========================================================================
// NOCTURNAL CLEARING (BGRAD.PNG)
// =========================================================================

pub struct ForestBackground {
    pub orig_w: usize,
    pub orig_h: usize,
    pub orig_pixels: Vec<u32>,
    pub scaled_w: usize,
    pub scaled_h: usize,
    pub scaled_pixels: Vec<u32>,
}

impl ForestBackground {
    pub fn load_or_generate(target_w: usize, target_h: usize) -> Self {
        let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/BGrad.png");

        let (orig_w, orig_h, orig_pixels) = if Path::new(asset_path).exists() {
            if let Ok(img) = image::open(asset_path) {
                let rgb = img.to_rgba8();
                let (w, h) = rgb.dimensions();
                let mut pixels = Vec::with_capacity((w * h) as usize);
                for p in rgb.pixels() {
                    let r = p[0] as u32;
                    let g = p[1] as u32;
                    let b = p[2] as u32;
                    pixels.push((r << 16) | (g << 8) | b);
                }
                (w as usize, h as usize, pixels)
            } else {
                Self::generate_fallback_master()
            }
        } else {
            Self::generate_fallback_master()
        };

        let mut bg = Self {
            orig_w,
            orig_h,
            orig_pixels,
            scaled_w: 0,
            scaled_h: 0,
            scaled_pixels: Vec::new(),
        };

        bg.rescale(target_w, target_h);
        bg
    }

    fn generate_fallback_master() -> (usize, usize, Vec<u32>) {
        let w = 1920;
        let h = 1080;
        let mut pixels = Vec::with_capacity(w * h);
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for y in 0..h {
            let dy = y as f32 - cy;
            for x in 0..w {
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy * dy).sqrt() / max_dist;
                let dither = (((x ^ (y * 57)) & 7) as f32 / 7.0 - 0.5) * 1.5;
                let vignette = dist.powf(1.8);
                let r = ((2.0 + 4.0 * vignette + dither).clamp(1.0, 9.0)) as u32;
                let g = ((3.0 + 7.0 * vignette + dither).clamp(2.0, 14.0)) as u32;
                let b = ((4.0 + 9.0 * vignette + dither).clamp(3.0, 16.0)) as u32;
                pixels.push((r << 16) | (g << 8) | b);
            }
        }
        (w, h, pixels)
    }

    pub fn rescale(&mut self, target_w: usize, target_h: usize) {
        if self.scaled_w == target_w && self.scaled_h == target_h && !self.scaled_pixels.is_empty() {
            return;
        }

        let mut scaled = vec![0u32; target_w * target_h];

        for y in 0..target_h {
            let src_y = (y * self.orig_h) / target_h;
            let row_dst = y * target_w;
            let row_src = src_y * self.orig_w;

            for x in 0..target_w {
                let src_x = (x * self.orig_w) / target_w;
                scaled[row_dst + x] = self.orig_pixels[row_src + src_x];
            }
        }

        self.scaled_w = target_w;
        self.scaled_h = target_h;
        self.scaled_pixels = scaled;
    }

    #[inline(always)]
    pub fn blit_with_alpha(&self, buffer: &mut [u32], dst_w: usize, dst_h: usize, alpha: f32) {
        let a = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
        if a == 0 {
            buffer.fill(0x00000000);
            return;
        }

        let len = (dst_w * dst_h).min(self.scaled_pixels.len()).min(buffer.len());

        if a >= 255 {
            buffer[..len].copy_from_slice(&self.scaled_pixels[..len]);
            return;
        }

        for i in 0..len {
            let src = self.scaled_pixels[i];
            let sr = (src >> 16) & 0xFF;
            let sg = (src >> 8) & 0xFF;
            let sb = src & 0xFF;

            let r = (sr * a) / 255;
            let g = (sg * a) / 255;
            let b = (sb * a) / 255;

            buffer[i] = (r << 16) | (g << 8) | b;
        }
    }
}

// =========================================================================
// PROCEDURAL HUD BUTTON ENGINE
// =========================================================================

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UiIcon {
    Help,        // [ ? ]
    Shuffle,     // [ ~ ]
    AudioNoise,  // [ ⑉ ] 3-Bar Spectral Falloff
    Exit,        // [ X ]
    Minimize,    // [ - ]
    Maximize,    // [ # ]
}

pub struct CssButton {
    pub x: isize,
    pub y: isize,
    pub w: usize,
    pub h: usize,
    pub icon: UiIcon,
    pub hovered: bool,
    pub active: bool,
}

impl CssButton {
    pub fn new(x: isize, y: isize, w: usize, h: usize, icon: UiIcon) -> Self {
        Self { x, y, w, h, icon, hovered: false, active: false }
    }

    pub fn contains(&self, mx: isize, my: isize) -> bool {
        mx >= self.x && mx < (self.x + self.w as isize) && my >= self.y && my < (self.y + self.h as isize)
    }

    pub fn render(&self, buffer: &mut [u32], buf_w: usize, buf_h: usize, opacity: f32) {
        if opacity <= 0.01 { return; }

        let (bg_r, bg_g, bg_b, bg_a) = if self.active {
            (25, 30, 38, 220)
        } else if self.hovered {
            (18, 24, 32, 190)
        } else {
            (8, 10, 14, 140)
        };

        let (border_r, border_g, border_b) = match self.icon {
            UiIcon::Exit if self.hovered => (245, 35, 45),
            UiIcon::AudioNoise if self.active => (255, 194, 38),
            UiIcon::AudioNoise if self.hovered => (210, 220, 235),
            _ if self.hovered => (210, 220, 235),
            _ => (65, 75, 85),
        };

        let final_bg_a = (bg_a as f32 * opacity) as u32;
        let inv_bg_a = 255 - final_bg_a;
        let bx0 = self.x;
        let by0 = self.y;

        for py in 0..self.h as isize {
            let dy = by0 + py;
            if dy < 0 || dy >= buf_h as isize { continue; }
            let row_offset = (dy as usize) * buf_w;

            for px in 0..self.w as isize {
                let dx = bx0 + px;
                if dx < 0 || dx >= buf_w as isize { continue; }

                let is_corner = (px == 0 && py == 0)
                    || (px == (self.w as isize - 1) && py == 0)
                    || (px == 0 && py == (self.h as isize - 1))
                    || (px == (self.w as isize - 1) && py == (self.h as isize - 1));

                if is_corner { continue; }

                let is_border = px == 0 || px == (self.w as isize - 1) || py == 0 || py == (self.h as isize - 1);
                let idx = row_offset + dx as usize;

                if is_border {
                    let br = (border_r as f32 * opacity) as u32;
                    let bg = (border_g as f32 * opacity) as u32;
                    let bb = (border_b as f32 * opacity) as u32;
                    buffer[idx] = (br << 16) | (bg << 8) | bb;
                } else {
                    let curr = buffer[idx];
                    let cr = (curr >> 16) & 0xFF;
                    let cg = (curr >> 8) & 0xFF;
                    let cb = curr & 0xFF;

                    let nr = (bg_r * final_bg_a + cr * inv_bg_a) / 255;
                    let ng = (bg_g * final_bg_a + cg * inv_bg_a) / 255;
                    let nb = (bg_b * final_bg_a + cb * inv_bg_a) / 255;
                    buffer[idx] = (nr << 16) | (ng << 8) | nb;
                }
            }
        }

        let (icon_r, icon_g, icon_b) = if self.hovered || (self.icon == UiIcon::AudioNoise && self.active) {
            (border_r, border_g, border_b)
        } else {
            (140, 150, 165)
        };
        self.render_icon_glyph(buffer, buf_w, buf_h, icon_r, icon_g, icon_b, opacity);
    }

    fn render_icon_glyph(&self, buffer: &mut [u32], buf_w: usize, buf_h: usize, r: u8, g: u8, b: u8, opacity: f32) {
        let col = (((r as f32 * opacity) as u32) << 16)
            | (((g as f32 * opacity) as u32) << 8)
            | ((b as f32 * opacity) as u32);

        let cx = self.x + (self.w as isize) / 2;
        let cy = self.y + (self.h as isize) / 2;

        let draw_pt = |buf: &mut [u32], px: isize, py: isize| {
            if px >= 0 && px < buf_w as isize && py >= 0 && py < buf_h as isize {
                buf[(py as usize) * buf_w + px as usize] = col;
            }
        };

        match self.icon {
            UiIcon::Exit => {
                for i in -4..=4 {
                    draw_pt(buffer, cx + i, cy + i);
                    draw_pt(buffer, cx + i, cy - i);
                }
            }
            UiIcon::AudioNoise => {
                // 3-Bar Spectral Falloff [ ⑉ ]
                let base_y = cy + 4;
                for y in (base_y - 8)..=base_y { draw_pt(buffer, cx - 4, y); }
                for y in (base_y - 5)..=base_y { draw_pt(buffer, cx, y); }
                for y in (base_y - 2)..=base_y { draw_pt(buffer, cx + 4, y); }
            }
            UiIcon::Help => {
                for i in -2..=2 { draw_pt(buffer, cx + i, cy - 4); }
                draw_pt(buffer, cx + 2, cy - 3);
                draw_pt(buffer, cx + 2, cy - 2);
                draw_pt(buffer, cx + 1, cy - 1);
                draw_pt(buffer, cx, cy);
                draw_pt(buffer, cx, cy + 1);
                draw_pt(buffer, cx, cy + 4);
            }
            UiIcon::Shuffle => {
                for i in -4..=-2 { draw_pt(buffer, cx + i, cy - 2); }
                draw_pt(buffer, cx - 1, cy - 1);
                draw_pt(buffer, cx, cy);
                draw_pt(buffer, cx + 1, cy + 1);
                for i in 2..=4 { draw_pt(buffer, cx + i, cy + 2); }
            }
            UiIcon::Minimize => {
                for i in -4..=4 { draw_pt(buffer, cx + i, cy + 2); }
            }
            UiIcon::Maximize => {
                for i in -4..=4 {
                    draw_pt(buffer, cx + i, cy - 4);
                    draw_pt(buffer, cx + i, cy + 4);
                    draw_pt(buffer, cx - 4, cy + i);
                    draw_pt(buffer, cx + 4, cy + i);
                }
            }
        }
    }
}

// =========================================================================
// ENMEI FLASH SHUTDOWN IMAGE
// =========================================================================

pub struct EnmeiFlashImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl EnmeiFlashImage {
    pub fn load() -> Self {
        const ENMEI_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/enmei.png"));
        let img = image::load_from_memory_with_format(ENMEI_BYTES, image::ImageFormat::Png)
            .expect("Critical failure loading assets/enmei.png")
            .to_rgba8();

        let (w, h) = img.dimensions();
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for p in img.pixels() {
            let r = p[0] as u32;
            let g = p[1] as u32;
            let b = p[2] as u32;
            let a = p[3] as u32;
            pixels.push((a << 24) | (r << 16) | (g << 8) | b);
        }

        Self { width: w as usize, height: h as usize, pixels }
    }
}

// =========================================================================
// CPU RASTER PRIMITIVES
// =========================================================================

#[inline(always)]
fn blend_add(buffer: &mut [u32], idx: usize, r: f32, g: f32, b: f32) {
    if idx >= buffer.len() { return; }
    let curr = buffer[idx];
    let cr = ((curr >> 16) & 0xFF) as f32;
    let cg = ((curr >> 8) & 0xFF) as f32;
    let cb = (curr & 0xFF) as f32;

    let nr = (cr + r).min(255.0) as u32;
    let ng = (cg + g).min(255.0) as u32;
    let nb = (cb + b).min(255.0) as u32;

    buffer[idx] = (nr << 16) | (ng << 8) | nb;
}

#[inline(always)]
fn splat_bilinear(buffer: &mut [u32], width: usize, height: usize, x: f32, y: f32, r: f32, g: f32, b: f32) {
    if x < 0.0 || x >= (width as f32 - 1.0) || y < 0.0 || y >= (height as f32 - 1.0) { return; }

    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let fx = x - x.floor();
    let fy = y - y.floor();

    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    blend_add(buffer, y0 * width + x0, r * w00, g * w00, b * w00);
    blend_add(buffer, y0 * width + x1, r * w10, g * w10, b * w10);
    blend_add(buffer, y1 * width + x0, r * w01, g * w01, b * w01);
    blend_add(buffer, y1 * width + x1, r * w11, g * w11, b * w11);
}

#[inline(always)]
pub fn project(p: Vec3, cx: f32, cy: f32, focal: f32) -> Option<(f32, f32, f32)> {
    let z_cam = CAMERA_DIST - p.z;
    if z_cam <= 0.1 { return None; }
    let inv_z = 1.0 / z_cam;
    let sx = cx + p.x * focal * inv_z;
    let sy = cy - p.y * focal * inv_z;
    let z_norm = ((p.z + 1.0) * 0.5).clamp(0.0, 1.0);
    Some((sx, sy, z_norm))
}

fn render_thin_segment(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    p0: (f32, f32, f32),
    p1: (f32, f32, f32),
    base_r: f32,
    base_g: f32,
    base_b: f32,
    alpha_mult: f32,
) {
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let len = (dx * dx + dy * dy).sqrt();

    if len < 0.1 {
        let z_fade = 0.20 + 0.80 * p0.2;
        let a = alpha_mult * z_fade;
        splat_bilinear(buffer, width, height, p0.0, p0.1, base_r * a, base_g * a, base_b * a);
        return;
    }

    let steps = (len * 1.5).ceil() as usize;
    let inv_steps = 1.0 / steps as f32;

    for s in 0..=steps {
        let t = s as f32 * inv_steps;
        let x = p0.0 + dx * t;
        let y = p0.1 + dy * t;
        let z = p0.2 + (p1.2 - p0.2) * t;

        let z_fade = (0.15 + 0.85 * z).powf(1.4);
        let a = alpha_mult * z_fade;

        splat_bilinear(buffer, width, height, x, y, base_r * a, base_g * a, base_b * a);
    }
}

// =========================================================================
// RADIANT MAIN RENDERER
// =========================================================================

pub struct RadiantRenderer;

impl RadiantRenderer {
    pub fn render(
        background: &ForestBackground,
        lattice: &CelestialLattice,
        arcball: &Arcball,
        time: f32,
        emergence: f32,
        buttons: &[CssButton],
        logo: &EnmeiLogoImage,
        buffer: &mut [u32],
        width: usize,
        height: usize,
    ) {
        let cx = arcball.center_x;
        let cy = arcball.center_y;
        let orientation = arcball.orientation;

        let sphere_radius = (height as f32) * 0.30;
        let focal_length = sphere_radius * CAMERA_DIST;

        let bg_alpha = (emergence / 0.6).clamp(0.0, 1.0);
        let orb_alpha = ((emergence - 0.5) / 0.5).clamp(0.0, 1.0);

        background.blit_with_alpha(buffer, width, height, bg_alpha);

        if orb_alpha > 0.001 {
            Self::render_crystalline_fresnel(buffer, width, height, cx, cy, sphere_radius, time, orb_alpha);

            // Volumetric dust swarm
            for particle in &lattice.dust {
                let rot_angle = particle.speed * time * 0.20;
                let cos_r = rot_angle.cos();
                let sin_r = rot_angle.sin();
                let drifted = Vec3::new(
                    particle.pos.x * cos_r - particle.pos.z * sin_r,
                    particle.pos.y + 0.03 * (time * 1.5 + particle.phase).sin(),
                    particle.pos.x * sin_r + particle.pos.z * cos_r,
                );

                let rot_pos = orientation.rotate_vec(drifted);
                if let Some((sx, sy, z_norm)) = project(rot_pos, cx, cy, focal_length) {
                    let scint = 0.35 + 0.65 * (particle.phase + time * particle.speed * 3.0).sin().abs();
                    let z_fade = (0.20 + 0.80 * z_norm).powf(1.2);
                    let intensity = particle.intensity * scint * z_fade * 0.70 * orb_alpha;

                    let (pr, pg, pb) = if particle.is_gold {
                        (255.0, 205.0, 75.0)
                    } else {
                        (120.0, 190.0, 255.0)
                    };

                    splat_bilinear(buffer, width, height, sx, sy, pr * intensity, pg * intensity, pb * intensity);
                }
            }

            // Geodesic mesh
            for synapse in &lattice.synapses {
                let mut prev_proj: Option<(f32, f32, f32)> = None;

                for sample in &synapse.samples {
                    let rot_pos = orientation.rotate_vec(*sample);
                    let current_proj = project(rot_pos, cx, cy, focal_length);

                    if let (Some(p0), Some(p1)) = (prev_proj, current_proj) {
                        render_thin_segment(
                            buffer,
                            width,
                            height,
                            p0,
                            p1,
                            COLOR_LATTICE_R,
                            COLOR_LATTICE_G,
                            COLOR_LATTICE_B,
                            0.13 * orb_alpha,
                        );
                    }
                    prev_proj = current_proj;
                }
            }

            // Orbital rings
            for (ring_idx, ring) in lattice.rings.iter().enumerate() {
                let ring_rot = Quat::from_axis_angle(ring.axis_tilt, time * ring.rot_speed);

                let (cr, cg, cb) = if ring_idx == 0 {
                    (255.0, 160.0, 45.0)
                } else if ring_idx == 2 {
                    (255.0, 215.0, 85.0)
                } else {
                    (235.0, 200.0, 120.0)
                };

                for i in 0..RING_SAMPLES {
                    let p0_raw = ring.base_samples[i] * ring.radius;
                    let p1_raw = ring.base_samples[(i + 1) % RING_SAMPLES] * ring.radius;

                    let p0_rot = orientation.rotate_vec(ring_rot.rotate_vec(p0_raw));
                    let p1_rot = orientation.rotate_vec(ring_rot.rotate_vec(p1_raw));

                    let theta = (i as f32 / RING_SAMPLES as f32) * std::f32::consts::PI * 8.0;
                    let wave = (theta - time * 2.5).sin().powi(2) * 0.40 + 0.35;

                    if let (Some(proj0), Some(proj1)) = (project(p0_rot, cx, cy, focal_length), project(p1_rot, cx, cy, focal_length)) {
                        let z_avg = (proj0.2 + proj1.2) * 0.5;
                        let ring_fade = if z_avg < 0.45 { 0.25 } else { 0.75 };
                        render_thin_segment(buffer, width, height, proj0, proj1, cr, cg, cb, wave * ring_fade * orb_alpha);
                    }
                }
            }

            // All 20 celestial star beacons (Pure Solar Gold)
            for star in &lattice.stars {
                let rot_pos = orientation.rotate_vec(star.pos);
                if let Some((sx, sy, z_norm)) = project(rot_pos, cx, cy, focal_length) {
                    let pulse = 0.90 + 0.25 * (time * 2.4 + star.phase).sin();
                    Self::render_massive_star_beacon(buffer, width, height, sx, sy, pulse, z_norm, orb_alpha);
                }
            }
        }

        // HUD Buttons
        for btn in buttons {
            btn.render(buffer, width, height, orb_alpha);
        }

        // Monogram
        let logo_x = width.saturating_sub(logo.width + 24);
        let logo_y = height.saturating_sub(logo.height + 18);
        logo.render_pixel_perfect(buffer, width, height, logo_x, logo_y, orb_alpha * 0.85);
    }

    fn render_massive_star_beacon(
        buffer: &mut [u32],
        width: usize,
        height: usize,
        sx: f32,
        sy: f32,
        scale: f32,
        z_norm: f32,
        alpha: f32,
    ) {
        let is_front = z_norm > 0.40;
        let depth_scale = if is_front { 1.25 } else { 0.65 };
        let final_scale = scale * depth_scale * alpha;

        let (core_tint_r, core_tint_g, core_tint_b) = (255.0, 165.0, 8.0);  // Imperial Gold
        let (halo_r, halo_g, halo_b) = (255.0, 140.0, 0.0);
        let (spike_r, spike_g, spike_b) = (255.0, 180.0, 20.0);

        let core_radius = if is_front { 5.8 } else { 3.2 };
        let r_sq_max = core_radius * core_radius;

        let min_x = (sx - core_radius - 1.0).floor() as isize;
        let max_x = (sx + core_radius + 1.0).ceil() as isize;
        let min_y = (sy - core_radius - 1.0).floor() as isize;
        let max_y = (sy + core_radius + 1.0).ceil() as isize;

        // Core
        for y in min_y..=max_y {
            if y < 0 || y >= height as isize { continue; }
            let dy = y as f32 - sy;
            let dy_sq = dy * dy;

            for x in min_x..=max_x {
                if x < 0 || x >= width as isize { continue; }
                let dx = x as f32 - sx;
                let d_sq = dx * dx + dy_sq;

                if d_sq <= r_sq_max {
                    let w = (1.0 - (d_sq / r_sq_max)).powf(1.4) * final_scale;
                    blend_add(buffer, (y as usize) * width + (x as usize), core_tint_r * w, core_tint_g * w, core_tint_b * w);
                }
            }
        }

        // Halo
        let hr = halo_r * final_scale * 0.70;
        let hg = halo_g * final_scale * 0.70;
        let hb = halo_b * final_scale * 0.70;

        for r in 3..=14 {
            let rf = r as f32;
            let fade = (1.0 - rf / 15.0).powf(1.3) * 0.60;
            splat_bilinear(buffer, width, height, sx + rf, sy, hr * fade, hg * fade, hb * fade);
            splat_bilinear(buffer, width, height, sx - rf, sy, hr * fade, hg * fade, hb * fade);
            splat_bilinear(buffer, width, height, sx, sy + rf, hr * fade, hg * fade, hb * fade);
            splat_bilinear(buffer, width, height, sx, sy - rf, hr * fade, hg * fade, hb * fade);
        }

        // Diffraction Spikes
        if is_front {
            let spike_len = (22.0 * scale) as usize;
            for s in 4..=spike_len {
                let sf = s as f32;
                let spike_fade = (1.0 - sf / (spike_len as f32 + 1.0)).powi(2) * 0.75 * alpha;
                splat_bilinear(buffer, width, height, sx + sf, sy, spike_r * spike_fade, spike_g * spike_fade, spike_b * spike_fade);
                splat_bilinear(buffer, width, height, sx - sf, sy, spike_r * spike_fade, spike_g * spike_fade, spike_b * spike_fade);
                splat_bilinear(buffer, width, height, sx, sy + sf, spike_r * spike_fade, spike_g * spike_fade, spike_b * spike_fade);
                splat_bilinear(buffer, width, height, sx, sy - sf, spike_r * spike_fade, spike_g * spike_fade, spike_b * spike_fade);
            }
        }
    }

    fn render_crystalline_fresnel(
        buffer: &mut [u32],
        width: usize,
        height: usize,
        cx: f32,
        cy: f32,
        radius: f32,
        time: f32,
        alpha: f32,
    ) {
        let r_max = radius + 3.0;
        let r_sq_max = r_max * r_max;
        let r_sq_inner = radius * radius;

        let min_x = ((cx - r_max).floor() as isize).max(0) as usize;
        let max_x = ((cx + r_max).ceil() as isize).min(width as isize - 1) as usize;
        let min_y = ((cy - r_max).floor() as isize).max(0) as usize;
        let max_y = ((cy + r_max).ceil() as isize).min(height as isize - 1) as usize;

        let breath = (0.45 + 0.05 * (time * 1.8).sin()) * alpha;

        for y in min_y..=max_y {
            let dy = y as f32 - cy;
            let dy_sq = dy * dy;
            let row_offset = y * width;

            for x in min_x..=max_x {
                let dx = x as f32 - cx;
                let dist_sq = dx * dx + dy_sq;
                if dist_sq > r_sq_max { continue; }

                let intensity = if dist_sq <= r_sq_inner {
                    let u = (dist_sq / r_sq_inner).sqrt();
                    u.powi(14) * breath
                } else {
                    let d = dist_sq.sqrt();
                    let fade = (1.0 - (d - radius) / 3.0).max(0.0);
                    breath * fade
                };

                if intensity > 0.005 {
                    blend_add(buffer, row_offset + x, COLOR_FRESNEL_R * intensity, COLOR_FRESNEL_G * intensity, COLOR_FRESNEL_B * intensity);
                }
            }
        }
    }

    pub fn render_crt_shutdown(
        buffer: &mut [u32],
        width: usize,
        height: usize,
        flash_img: &EnmeiFlashImage,
        elapsed: f32,
    ) {
        let center_y = height / 2;
        let center_x = width / 2;

        if elapsed < 0.08 {
            let progress = elapsed / 0.08;
            let half_h = (((height as f32) * 0.5 * (1.0 - progress * 0.85)) as usize).max(4);

            let y_start = center_y.saturating_sub(half_h);
            let y_end = (center_y + half_h).min(height);

            for y in 0..y_start {
                let off = y * width;
                buffer[off..(off + width)].fill(0x00000000);
            }
            for y in y_end..height {
                let off = y * width;
                buffer[off..(off + width)].fill(0x00000000);
            }

            let boost = 1.0 + progress * 0.4;
            for y in y_start..y_end {
                let row_offset = y * width;
                for x in 0..width {
                    let p = buffer[row_offset + x];
                    let r = (((p >> 16) & 0xFF) as f32 * boost).min(255.0) as u32;
                    let g = (((p >> 8) & 0xFF) as f32 * boost).min(255.0) as u32;
                    let b = ((p & 0xFF) as f32 * boost).min(255.0) as u32;
                    buffer[row_offset + x] = (r << 16) | (g << 8) | b;
                }
            }
        } else if elapsed < 0.46 {
            buffer.fill(0x00000000);

            let t = (elapsed - 0.08) / 0.38;
            let intensity = (t * std::f32::consts::PI).sin().powf(0.8);

            let dst_x0 = (width.saturating_sub(flash_img.width)) / 2;
            let dst_y0 = (height.saturating_sub(flash_img.height)) / 2;

            for py in 0..flash_img.height {
                let dy = dst_y0 + py;
                if dy >= height { break; }
                let row_offset = dy * width;
                let scanline_dim = if (dy % 2) == 0 { 0.92 } else { 1.0 };
                let final_mult = intensity * scanline_dim;

                for px in 0..flash_img.width {
                    let dx = dst_x0 + px;
                    if dx >= width { break; }

                    let p = flash_img.pixels[py * flash_img.width + px];
                    let a = (p >> 24) & 0xFF;
                    let r = (p >> 16) & 0xFF;
                    let g = (p >> 8) & 0xFF;
                    let b = p & 0xFF;

                    if a > 10 && (r | g | b) > 10 {
                        let nr = (r as f32 * final_mult).min(255.0) as u32;
                        let ng = (g as f32 * final_mult).min(255.0) as u32;
                        let nb = (b as f32 * final_mult).min(255.0) as u32;
                        buffer[row_offset + dx] = (nr << 16) | (ng << 8) | nb;
                    }
                }
            }
        } else if elapsed < 0.62 {
            buffer.fill(0x00000000);

            let progress = (elapsed - 0.46) / 0.16;
            let half_w = (((width as f32) * 0.5 * (1.0 - progress.powf(1.4))) as usize).max(1);

            let x_start = center_x.saturating_sub(half_w);
            let x_end = (center_x + half_w).min(width);

            for dy in 0..2 {
                let row_offset = (center_y - 1 + dy) * width;
                for x in x_start..x_end {
                    let dist = (x as f32 - center_x as f32).abs() / (half_w as f32);
                    let edge_fade = (1.0 - dist.powi(2)).max(0.0);

                    let r = (245.0 * edge_fade) as u32;
                    let g = (255.0 * edge_fade) as u32;
                    let b = (255.0 * edge_fade) as u32;
                    buffer[row_offset + x] = (r << 16) | (g << 8) | b;
                }
            }
        } else {
            buffer.fill(0x00000000);

            let decay_t = (elapsed - 0.62) / 0.04;
            let energy = (-decay_t).exp().max(0.0);

            for dy in -1..=1 {
                let y = (center_y as isize + dy) as usize;
                let row_offset = y * width;
                for dx in -1..=1 {
                    let x = (center_x as isize + dx) as usize;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    let falloff = (1.5 - dist_sq * 0.5).max(0.0) * energy;

                    let r = (255.0 * falloff) as u32;
                    let g = (250.0 * falloff) as u32;
                    let b = (230.0 * falloff) as u32;
                    buffer[row_offset + x] = (r << 16) | (g << 8) | b;
                }
            }
        }
    }
}