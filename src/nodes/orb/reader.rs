//! THE BLACK FOREST // NODE 06: TYPOGRAPHIC READER & GUIDE SANCTUARY
//!
//! Sovereign offline reading sanctuary:

use fontdue::{Font, FontSettings};
use crate::nodes::vault::VaultImage;

pub const PAPER_WIDTH: usize = 850;
pub const PAPER_HEIGHT: usize = 720;

struct ThemePalette {
    paper: u32,
    ink: u32,
    ink_muted: u32,
    border: u32,
    rule: u32,
    gold: u32,
}

const LIGHT_THEME: ThemePalette = ThemePalette {
    paper: 0x00F4F1EA,
    ink: 0x002B2824,
    ink_muted: 0x00686358,
    border: 0x003A362E,
    rule: 0x00D8D3C6,
    gold: 0x009C6600,
};

const DARK_THEME: ThemePalette = ThemePalette {
    paper: 0x00191A1D,
    ink: 0x00C5C1B8,
    ink_muted: 0x00838079,
    border: 0x0033353C,
    rule: 0x002A2B30,
    gold: 0x00D8A02A,
};

#[derive(Clone)]
struct FormattedWord {
    text: String,
    is_italic: bool,
}

#[derive(Clone)]
struct FormattedLine {
    words: Vec<FormattedWord>,
}

struct LoadedArticle {
    title: String,
    subtitle: String,
    pages_lines: Vec<Vec<FormattedLine>>,
    primary_image: Option<VaultImage>,
}

pub struct HelpReader {
    pub font_regular: Font,
    pub font_italic: Font,
    pub paper_light: Vec<u32>,
    pub paper_dark: Vec<u32>,
    pub is_dark_mode: bool,
    pub width: usize,
    pub height: usize,
    pub is_open: bool,
    pub is_article_view: bool,
    pub current_page: usize,
    pub total_pages: usize,
    loaded_article: Option<LoadedArticle>,
}

impl HelpReader {
    pub fn new() -> Self {
        const FONT_REGULAR_BYTES: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/Literata-Regular.ttf"
        ));
        const FONT_ITALIC_BYTES: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/Literata-Italic.ttf"
        ));

        let font_regular = Font::from_bytes(FONT_REGULAR_BYTES, FontSettings::default())
            .expect("Failed to load Literata-Regular.ttf");
        let font_italic = Font::from_bytes(FONT_ITALIC_BYTES, FontSettings::default())
            .expect("Failed to load Literata-Italic.ttf");

        let paper_light = Self::bake_guide_sheet(&font_regular, &font_italic, &LIGHT_THEME);
        let paper_dark = Self::bake_guide_sheet(&font_regular, &font_italic, &DARK_THEME);

        Self {
            font_regular,
            font_italic,
            paper_light,
            paper_dark,
            is_dark_mode: false,
            width: PAPER_WIDTH,
            height: PAPER_HEIGHT,
            is_open: false,
            is_article_view: false,
            current_page: 0,
            total_pages: 1,
            loaded_article: None,
        }
    }

    pub fn open_guide(&mut self) {
        self.paper_light = Self::bake_guide_sheet(&self.font_regular, &self.font_italic, &LIGHT_THEME);
        self.paper_dark = Self::bake_guide_sheet(&self.font_regular, &self.font_italic, &DARK_THEME);
        self.is_article_view = false;
        self.current_page = 0;
        self.total_pages = 1;
        self.loaded_article = None;
        self.is_open = true;
    }

    pub fn open_article(
        &mut self,
        title: &str,
        subtitle: &str,
        markdown: &str,
        image: Option<&VaultImage>,
    ) {
        let max_text_width = (PAPER_WIDTH - 104) as f32;
        let line_step = 23;

        let formatted_words = Self::parse_markdown_spans(markdown);
        let all_lines = Self::wrap_formatted_words(&self.font_regular, &self.font_italic, formatted_words, max_text_width, 14.5);

        let mut valid_image = false;
        if let Some(img_data) = image {
            if image::load_from_memory(&img_data.bytes).is_ok() {
                valid_image = true;
            }
        }

        let page1_line_count = if valid_image {
            ((PAPER_HEIGHT - 65).saturating_sub(390)) / line_step
        } else {
            ((PAPER_HEIGHT - 65).saturating_sub(128)) / line_step
        };

        let subsequent_line_count = ((PAPER_HEIGHT - 65).saturating_sub(128)) / line_step;

        let mut pages_lines = Vec::new();
        let mut line_cursor = 0;

        let p1_end = (line_cursor + page1_line_count).min(all_lines.len());
        pages_lines.push(all_lines[line_cursor..p1_end].to_vec());
        line_cursor = p1_end;

        while line_cursor < all_lines.len() {
            let next_end = (line_cursor + subsequent_line_count).min(all_lines.len());
            pages_lines.push(all_lines[line_cursor..next_end].to_vec());
            line_cursor = next_end;
        }

        if pages_lines.is_empty() {
            pages_lines.push(Vec::new());
        }

        self.total_pages = pages_lines.len();
        self.current_page = 0;

        self.loaded_article = Some(LoadedArticle {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            pages_lines,
            primary_image: if valid_image { image.cloned() } else { None },
        });

        self.is_article_view = true;
        self.is_open = true;
        self.bake_current_article_page();
    }

    pub fn next_page(&mut self) {
        if self.is_article_view && self.current_page + 1 < self.total_pages {
            self.current_page += 1;
            self.bake_current_article_page();
        }
    }

    pub fn prev_page(&mut self) {
        if self.is_article_view && self.current_page > 0 {
            self.current_page -= 1;
            self.bake_current_article_page();
        }
    }

    fn bake_current_article_page(&mut self) {
        if let Some(ref article) = self.loaded_article {
            self.paper_light = Self::bake_single_article_page(
                &self.font_regular,
                &self.font_italic,
                &LIGHT_THEME,
                article,
                self.current_page,
                self.total_pages,
            );
            self.paper_dark = Self::bake_single_article_page(
                &self.font_regular,
                &self.font_italic,
                &DARK_THEME,
                article,
                self.current_page,
                self.total_pages,
            );
        }
    }

    pub fn toggle_theme(&mut self) {
        self.is_dark_mode = !self.is_dark_mode;
    }

    fn parse_markdown_spans(raw: &str) -> Vec<FormattedWord> {
        let mut words = Vec::new();

        let mut text = String::with_capacity(raw.len());
        let chars: Vec<char> = raw.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '[' {
                if let Some(close_bracket) = chars[i..].iter().position(|&c| c == ']') {
                    let label_end = i + close_bracket;
                    if label_end + 1 < chars.len() && chars[label_end + 1] == '(' {
                        if let Some(close_paren) = chars[label_end + 1..].iter().position(|&c| c == ')') {
                            let url_end = label_end + 1 + close_paren;
                            for &lc in &chars[i + 1..label_end] {
                                text.push(lc);
                            }
                            i = url_end + 1;
                            continue;
                        }
                    }
                }
            }
            text.push(chars[i]);
            i += 1;
        }

        let normalized = text
            .replace("<it>", " *IT_START* ")
            .replace("</it>", " *IT_END* ")
            .replace("<i>", " *IT_START* ")
            .replace("</i>", " *IT_END* ");

        let mut in_italic = false;

        for line in normalized.lines() {
            let mut trimmed = line.trim();
            while trimmed.starts_with('#') {
                trimmed = trimmed.trim_start_matches('#').trim();
            }

            if trimmed.is_empty() {
                words.push(FormattedWord { text: "\n".to_string(), is_italic: false });
                continue;
            }

            for token in trimmed.split_whitespace() {
                if token == "*IT_START*" {
                    in_italic = true;
                    continue;
                }
                if token == "*IT_END*" {
                    in_italic = false;
                    continue;
                }

                let is_bold = token.starts_with("**") && token.ends_with("**") && token.len() >= 4;
                let clean_token = if is_bold {
                    &token[2..token.len() - 2]
                } else if token.starts_with('*') && token.ends_with('*') && token.len() >= 2 {
                    let inner = &token[1..token.len() - 1];
                    words.push(FormattedWord { text: inner.to_string(), is_italic: true });
                    continue;
                } else {
                    token
                };

                words.push(FormattedWord {
                    text: clean_token.to_string(),
                    is_italic: in_italic,
                });
            }

            words.push(FormattedWord { text: "\n".to_string(), is_italic: false });
        }

        words
    }

    fn wrap_formatted_words(
        font_regular: &Font,
        font_italic: &Font,
        words: Vec<FormattedWord>,
        max_w: f32,
        px: f32,
    ) -> Vec<FormattedLine> {
        let mut lines = Vec::new();
        let mut current_line = FormattedLine { words: Vec::new() };
        let mut current_w = 0.0f32;
        let space_w = font_regular.metrics(' ', px).advance_width;

        for w in words {
            if w.text == "\n" {
                if !current_line.words.is_empty() {
                    lines.push(current_line);
                    current_line = FormattedLine { words: Vec::new() };
                    current_w = 0.0;
                }
                lines.push(FormattedLine { words: Vec::new() });
                continue;
            }

            let font = if w.is_italic { font_italic } else { font_regular };
            let mut word_w = 0.0f32;
            for ch in w.text.chars() {
                word_w += font.metrics(ch, px).advance_width;
            }

            if current_line.words.is_empty() {
                current_line.words.push(w);
                current_w = word_w;
            } else if current_w + space_w + word_w <= max_w {
                current_line.words.push(w);
                current_w += space_w + word_w;
            } else {
                lines.push(current_line);
                current_line = FormattedLine { words: vec![w] };
                current_w = word_w;
            }
        }

        if !current_line.words.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    fn bake_single_article_page(
        font_regular: &Font,
        font_italic: &Font,
        pal: &ThemePalette,
        article: &LoadedArticle,
        page_idx: usize,
        total_pages: usize,
    ) -> Vec<u32> {
        let mut buffer = vec![pal.paper; PAPER_WIDTH * PAPER_HEIGHT];

        Self::draw_sheet_border(&mut buffer, pal.border);

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, "LOCAL VAULT ARCHIVE", 52, 44, 12.0, pal.gold);

        let clean_title_raw = article.title
            .replace("<it>", " *IT_START* ")
            .replace("</it>", " *IT_END* ")
            .replace("<i>", " *IT_START* ")
            .replace("</i>", " *IT_END* ");

        let mut title_pen_x = 52.0f32;
        let title_space_w = font_regular.metrics(' ', 20.0).advance_width;
        let mut title_in_italic = false;

        for token in clean_title_raw.split_whitespace() {
            if token == "*IT_START*" {
                title_in_italic = true;
                continue;
            }
            if token == "*IT_END*" {
                title_in_italic = false;
                continue;
            }

            let font = if title_in_italic { font_italic } else { font_regular };
            Self::draw_text(&mut buffer, PAPER_WIDTH, font, token, title_pen_x.round() as usize, 70, 20.0, pal.ink);
            
            for ch in token.chars() {
                title_pen_x += font.metrics(ch, 20.0).advance_width;
            }
            title_pen_x += title_space_w;
        }

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_italic, &article.subtitle, 52, 92, 12.5, pal.ink_muted);

        for x in 52..(PAPER_WIDTH - 52) {
            buffer[104 * PAPER_WIDTH + x] = pal.rule;
        }

        let mut y_cursor = 128;

        if page_idx == 0 {
            if let Some(ref img_meta) = article.primary_image {
                y_cursor = Self::draw_illustration_box(
                    &mut buffer,
                    PAPER_WIDTH,
                    font_italic,
                    pal,
                    img_meta,
                    120,
                );
            }
        }

        let line_step = 23;
        if let Some(lines) = article.pages_lines.get(page_idx) {
            for line in lines {
                if y_cursor >= PAPER_HEIGHT - 65 {
                    break;
                }
                if line.words.is_empty() {
                    y_cursor += line_step / 2;
                    continue;
                }

                let mut pen_x = 52.0f32;
                let space_w = font_regular.metrics(' ', 14.5).advance_width;

                for (idx, word) in line.words.iter().enumerate() {
                    if idx > 0 {
                        pen_x += space_w;
                    }
                    let font = if word.is_italic { font_italic } else { font_regular };
                    Self::draw_text(&mut buffer, PAPER_WIDTH, font, &word.text, pen_x.round() as usize, y_cursor, 14.5, pal.ink);

                    for ch in word.text.chars() {
                        pen_x += font.metrics(ch, 14.5).advance_width;
                    }
                }

                y_cursor += line_step;
            }
        }

        Self::draw_pagination_footer(&mut buffer, font_regular, font_italic, pal, page_idx, total_pages);
        Self::draw_top_buttons(&mut buffer, pal);

        buffer
    }

    fn draw_illustration_box(
        buffer: &mut [u32],
        buf_w: usize,
        font_italic: &Font,
        pal: &ThemePalette,
        image_meta: &VaultImage,
        y0: usize,
    ) -> usize {
        let max_box_w = 460;
        let max_box_h = 230;

        let bytes_ref = &image_meta.bytes;
        let decoded = image::load_from_memory(bytes_ref);
        if let Ok(img) = decoded {
            let rgba = img.to_rgba8();
            let (orig_w, orig_h) = rgba.dimensions();

            if orig_w > 0 && orig_h > 0 {
                let scale = ((max_box_w as f32) / orig_w as f32)
                    .min((max_box_h as f32) / orig_h as f32)
                    .min(1.0);

                let dst_w = (orig_w as f32 * scale).round() as usize;
                let dst_h = (orig_h as f32 * scale).round() as usize;
                let dst_x = (PAPER_WIDTH.saturating_sub(dst_w)) / 2;
                let dst_y = y0;

                for x in dst_x..(dst_x + dst_w) {
                    buffer[(dst_y.saturating_sub(1)) * buf_w + x] = pal.border;
                    buffer[(dst_y + dst_h) * buf_w + x] = pal.border;
                }
                for y in dst_y..(dst_y + dst_h) {
                    buffer[y * buf_w + dst_x.saturating_sub(1)] = pal.border;
                    buffer[y * buf_w + dst_x + dst_w] = pal.border;
                }

                for iy in 0..dst_h {
                    let sy = (iy * orig_h as usize) / dst_h;
                    let row_dst = (dst_y + iy) * buf_w;

                    for ix in 0..dst_w {
                        let sx = (ix * orig_w as usize) / dst_w;
                        let p = rgba.get_pixel(sx as u32, sy as u32);
                        let r = p[0] as u32;
                        let g = p[1] as u32;
                        let b = p[2] as u32;
                        buffer[row_dst + dst_x + ix] = (r << 16) | (g << 8) | b;
                    }
                }

                let mut cur_y = dst_y + dst_h + 18;

                if !image_meta.caption.is_empty() {
                    let clean_cap = image_meta.caption
                        .replace("<it>", "")
                        .replace("</it>", "")
                        .replace("<i>", "")
                        .replace("</i>", "");

                    let cap_w = (PAPER_WIDTH - 120) as f32;
                    let cap_lines = Self::wrap_raw_text(font_italic, &clean_cap, cap_w, 11.5);
                    for cline in cap_lines.iter().take(2) {
                        Self::draw_text(buffer, buf_w, font_italic, cline, 60, cur_y, 11.5, pal.ink_muted);
                        cur_y += 16;
                    }
                }

                return cur_y + 12;
            }
        }

        y0 + 8
    }

    fn wrap_raw_text(font: &Font, text: &str, max_w: f32, px: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let space_w = font.metrics(' ', px).advance_width;

        for para in text.lines() {
            let mut current = String::new();
            let mut cur_w = 0.0f32;

            for word in para.split_whitespace() {
                let mut word_w = 0.0f32;
                for ch in word.chars() {
                    word_w += font.metrics(ch, px).advance_width;
                }

                if current.is_empty() {
                    current.push_str(word);
                    cur_w = word_w;
                } else if cur_w + space_w + word_w <= max_w {
                    current.push(' ');
                    current.push_str(word);
                    cur_w += space_w + word_w;
                } else {
                    lines.push(current);
                    current = word.to_string();
                    cur_w = word_w;
                }
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }
        lines
    }

    fn draw_pagination_footer(
        buffer: &mut [u32],
        font_regular: &Font,
        font_italic: &Font,
        pal: &ThemePalette,
        page_idx: usize,
        total_pages: usize,
    ) {
        for x in 52..(PAPER_WIDTH - 52) {
            buffer[(PAPER_HEIGHT - 48) * PAPER_WIDTH + x] = pal.rule;
        }

        let footer_y = PAPER_HEIGHT - 22;

        if total_pages > 1 {
            let prev_col = if page_idx > 0 { pal.ink } else { pal.rule };
            Self::draw_text(buffer, PAPER_WIDTH, font_regular, "‹ PREV", 56, footer_y, 12.5, prev_col);

            let page_str = format!("Page {} of {}", page_idx + 1, total_pages);
            let cx = (PAPER_WIDTH / 2) - 36;
            Self::draw_text(buffer, PAPER_WIDTH, font_italic, &page_str, cx, footer_y, 12.0, pal.ink_muted);

            let next_col = if page_idx + 1 < total_pages { pal.ink } else { pal.rule };
            Self::draw_text(buffer, PAPER_WIDTH, font_regular, "NEXT ›", PAPER_WIDTH - 110, footer_y, 12.5, next_col);
        } else {
            Self::draw_text(
                buffer,
                PAPER_WIDTH,
                font_italic,
                "To close: click [ ? ], click [ ✕ ], press Esc, or click outside the sheet.",
                52,
                footer_y,
                12.0,
                pal.ink_muted,
            );
        }
    }

    fn bake_guide_sheet(font_regular: &Font, font_italic: &Font, pal: &ThemePalette) -> Vec<u32> {
        let mut buffer = vec![pal.paper; PAPER_WIDTH * PAPER_HEIGHT];

        Self::draw_sheet_border(&mut buffer, pal.border);

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, "THE BLACK FOREST", 52, 48, 22.0, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_italic, "Autonomous Sovereign Knowledge Sanctuary", 52, 72, 12.5, pal.ink_muted);

        for x in 52..(PAPER_WIDTH - 52) {
            buffer[86 * PAPER_WIDTH + x] = pal.rule;
        }

        let max_text_width = (PAPER_WIDTH - 104) as f32;

        let sec1_title = "1. PURPOSE & AUDIENCE";
        let sec1_body = "Built for students, children, researchers, and lifelong scholars. The Black Forest is a sovereign digital archive engineered to operate in complete isolation: emitting zero network telemetry, requiring no servers, and functioning perpetually offline. Thirty thousand fundamental articles and sixteen thousand historical image plates are sealed permanently on your machine.";

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, sec1_title, 52, 114, 14.0, pal.ink);
        let y_s1 = Self::draw_wrapped_text_simple(&mut buffer, PAPER_WIDTH, font_regular, sec1_body, 52, 138, max_text_width, 14.5, 1.58, pal.ink);

        let sec2_title = "2. ARCHITECTURAL INVULNERABILITY (DEEP SPACE GRADE)";
        let sec2_body = "Why is this software virtually indestructible across decades?
• CCSDS Space Telemetry Protection: Data blocks are armored with Reed-Solomon RS(255, 223) forward error correction—the spatial standard developed for deep space missions like voyager 1, 2, Rosetta and more. Even if underlying drive sectors degrade, the engine self-heals in memory.
• Cryptographic Armor: Hardened with Argon2id memory-hard key derivation and ChaCha20-Poly1305 authenticated encryption.
• Pure-CPU Driverless Invariance: All 3D raytracing and typography render directly on the CPU into a flat linear memory buffer. It avoids GPU shader deprecation, Vulkan/DirectX driver breaking changes, and web browser bloat. It executes on bare silicon (x64, ARM64, RISC-V)";

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, sec2_title, 52, y_s1 + 20, 14.0, pal.ink);
        let y_s2 = Self::draw_wrapped_text_simple(&mut buffer, PAPER_WIDTH, font_regular, sec2_body, 52, y_s1 + 44, max_text_width, 13.5, 1.50, pal.ink);

        let sec3_title = "3. INTERFACE CONTROLS";
        let ctrl_1 = "• Left-Click + Drag: Freely rotate the celestial orb in 3D space.";
        let ctrl_2 = "• Click any Star: Open the article and illustration plate in the sanctuary.";
        let ctrl_3 = "• [ ~ ] Shuffle: Draw a fresh constellation of 20 vault articles.";
        let ctrl_4 = "• [ ⑉ ] Noise Toggle: Toggle the analog Brownian ambient sound generator (1/f²).";
        let ctrl_5 = "• [ ◐ ] Contrast Toggle: Switch between warm book paper (Day) and slate ink (Night).";
        let ctrl_6 = "• [ ‹ PREV ] / [ NEXT › ] (or Arrow Keys ← / →): Turn pages of multi-page articles.";

        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, sec3_title, 52, y_s2 + 20, 14.0, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_1, 58, y_s2 + 44, 13.5, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_2, 58, y_s2 + 64, 13.5, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_3, 58, y_s2 + 84, 13.5, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_4, 58, y_s2 + 104, 13.5, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_5, 58, y_s2 + 124, 13.5, pal.ink);
        Self::draw_text(&mut buffer, PAPER_WIDTH, font_regular, ctrl_6, 58, y_s2 + 144, 13.5, pal.ink);

        Self::draw_pagination_footer(&mut buffer, font_regular, font_italic, pal, 0, 1);
        Self::draw_top_buttons(&mut buffer, pal);

        buffer
    }

    fn draw_sheet_border(buffer: &mut [u32], border_color: u32) {
        for x in 0..PAPER_WIDTH {
            buffer[x] = border_color;
            buffer[(PAPER_HEIGHT - 1) * PAPER_WIDTH + x] = border_color;
        }
        for y in 0..PAPER_HEIGHT {
            buffer[y * PAPER_WIDTH] = border_color;
            buffer[y * PAPER_WIDTH + (PAPER_WIDTH - 1)] = border_color;
        }
    }

    fn draw_top_buttons(buffer: &mut [u32], pal: &ThemePalette) {
        let cx = (PAPER_WIDTH - 42) as isize;
        let cy = 38isize;
        for i in -5..=5 {
            let p1 = (cy + i) * (PAPER_WIDTH as isize) + (cx + i);
            let p2 = (cy - i) * (PAPER_WIDTH as isize) + (cx + i);
            buffer[p1 as usize] = pal.border;
            buffer[p2 as usize] = pal.border;
        }

        let tx = (PAPER_WIDTH - 82) as isize;
        let ty = 38isize;
        for dy in -6..=6 {
            for dx in -6..=6 {
                let d2 = dx * dx + dy * dy;
                if (20..=36).contains(&d2) {
                    let p = (ty + dy) * (PAPER_WIDTH as isize) + (tx + dx);
                    buffer[p as usize] = pal.ink_muted;
                }
                if dx <= 0 && d2 <= 28 {
                    let p = (ty + dy) * (PAPER_WIDTH as isize) + (tx + dx);
                    buffer[p as usize] = pal.ink_muted;
                }
            }
        }
    }

    pub fn blit_to_framebuffer(&self, fb: &mut [u32], fb_w: usize, fb_h: usize) {
        if !self.is_open {
            return;
        }

        let dst_x = (fb_w.saturating_sub(self.width)) / 2;
        let dst_y = (fb_h.saturating_sub(self.height)) / 2;

        let shadow_margin = 16;
        let s_x0 = dst_x.saturating_sub(shadow_margin);
        let s_y0 = dst_y.saturating_sub(shadow_margin);
        let s_x1 = (dst_x + self.width + shadow_margin).min(fb_w);
        let s_y1 = (dst_y + self.height + shadow_margin).min(fb_h);

        for y in s_y0..s_y1 {
            let row = y * fb_w;
            for x in s_x0..s_x1 {
                if x >= dst_x && x < (dst_x + self.width) && y >= dst_y && y < (dst_y + self.height) {
                    continue;
                }
                let p = fb[row + x];
                let r = (((p >> 16) & 0xFF) * 30) / 100;
                let g = (((p >> 8) & 0xFF) * 30) / 100;
                let b = ((p & 0xFF) * 30) / 100;
                fb[row + x] = (r << 16) | (g << 8) | b;
            }
        }

        let active_sheet = if self.is_dark_mode {
            &self.paper_dark
        } else {
            &self.paper_light
        };

        for y in 0..self.height {
            let dy = dst_y + y;
            if dy >= fb_h {
                break;
            }
            let dst_offset = dy * fb_w + dst_x;
            let src_offset = y * self.width;
            let copy_w = self.width.min(fb_w - dst_x);
            fb[dst_offset..(dst_offset + copy_w)]
                .copy_from_slice(&active_sheet[src_offset..(src_offset + copy_w)]);
        }
    }

    pub fn is_inside_paper(&self, fb_w: usize, fb_h: usize, mx: isize, my: isize) -> bool {
        let dst_x = ((fb_w.saturating_sub(self.width)) / 2) as isize;
        let dst_y = ((fb_h.saturating_sub(self.height)) / 2) as isize;
        mx >= dst_x && mx < (dst_x + self.width as isize) && my >= dst_y && my < (dst_y + self.height as isize)
    }

    pub fn is_close_button_clicked(&self, fb_w: usize, fb_h: usize, mx: isize, my: isize) -> bool {
        let dst_x = ((fb_w.saturating_sub(self.width)) / 2) as isize;
        let dst_y = ((fb_h.saturating_sub(self.height)) / 2) as isize;
        let btn_x = dst_x + (self.width as isize) - 54;
        let btn_y = dst_y + 24;
        mx >= btn_x && mx < (btn_x + 32) && my >= btn_y && my < (btn_y + 32)
    }

    pub fn is_theme_button_clicked(&self, fb_w: usize, fb_h: usize, mx: isize, my: isize) -> bool {
        let dst_x = ((fb_w.saturating_sub(self.width)) / 2) as isize;
        let dst_y = ((fb_h.saturating_sub(self.height)) / 2) as isize;
        let btn_x = dst_x + (self.width as isize) - 96;
        let btn_y = dst_y + 24;
        mx >= btn_x && mx < (btn_x + 32) && my >= btn_y && my < (btn_y + 32)
    }

    pub fn is_prev_page_clicked(&self, fb_w: usize, fb_h: usize, mx: isize, my: isize) -> bool {
        if !self.is_article_view || self.total_pages <= 1 {
            return false;
        }
        let dst_x = ((fb_w.saturating_sub(self.width)) / 2) as isize;
        let dst_y = ((fb_h.saturating_sub(self.height)) / 2) as isize;
        let btn_x = dst_x + 50;
        let btn_y = dst_y + (self.height as isize) - 45;
        mx >= btn_x && mx < (btn_x + 65) && my >= btn_y && my < (btn_y + 40)
    }

    pub fn is_next_page_clicked(&self, fb_w: usize, fb_h: usize, mx: isize, my: isize) -> bool {
        if !self.is_article_view || self.total_pages <= 1 {
            return false;
        }
        let dst_x = ((fb_w.saturating_sub(self.width)) / 2) as isize;
        let dst_y = ((fb_h.saturating_sub(self.height)) / 2) as isize;
        let btn_x = dst_x + (self.width as isize) - 120;
        let btn_y = dst_y + (self.height as isize) - 45;
        mx >= btn_x && mx < (btn_x + 65) && my >= btn_y && my < (btn_y + 40)
    }

    fn draw_text(
        buffer: &mut [u32],
        buf_w: usize,
        font: &Font,
        text: &str,
        x0: usize,
        baseline_y: usize,
        px: f32,
        color: u32,
    ) {
        let mut pen_x = x0 as f32;

        let fg_r_lin = srgb_to_linear(((color >> 16) & 0xFF) as u8);
        let fg_g_lin = srgb_to_linear(((color >> 8) & 0xFF) as u8);
        let fg_b_lin = srgb_to_linear((color & 0xFF) as u8);

        for ch in text.chars() {
            if ch == '⑉' {
                let sigil_w = 14.0f32;
                let cx = (pen_x + sigil_w * 0.5).round() as isize;
                let base_y = (baseline_y as isize) - (px * 0.10).round() as isize;

                let draw_pt = |buf: &mut [u32], px_pos: isize, py_pos: isize| {
                    if px_pos >= 0 && px_pos < buf_w as isize && py_pos >= 0 && py_pos < PAPER_HEIGHT as isize {
                        buf[(py_pos as usize) * buf_w + px_pos as usize] = color;
                    }
                };

                for y in (base_y - 8)..=base_y { draw_pt(buffer, cx - 4, y); }
                for y in (base_y - 5)..=base_y { draw_pt(buffer, cx, y); }
                for y in (base_y - 2)..=base_y { draw_pt(buffer, cx + 4, y); }

                pen_x += sigil_w + 3.0;
                continue;
            }

            if ch == '✕' {
                let sigil_w = 12.0f32;
                let cx = (pen_x + sigil_w * 0.5).round() as isize;
                let cy = (baseline_y as isize) - (px * 0.32).round() as isize;

                for i in -4..=4 {
                    let p1_x = cx + i;
                    let p1_y = cy + i;
                    let p2_x = cx + i;
                    let p2_y = cy - i;
                    if p1_x >= 0 && p1_x < buf_w as isize && p1_y >= 0 && p1_y < PAPER_HEIGHT as isize {
                        buffer[(p1_y as usize) * buf_w + p1_x as usize] = color;
                    }
                    if p2_x >= 0 && p2_x < buf_w as isize && p2_y >= 0 && p2_y < PAPER_HEIGHT as isize {
                        buffer[(p2_y as usize) * buf_w + p2_x as usize] = color;
                    }
                }

                pen_x += sigil_w + 3.0;
                continue;
            }

            if ch == '◐' {
                let sigil_w = 14.0f32;
                let sigil_cx = (pen_x + sigil_w * 0.5).round() as isize;
                let sigil_cy = (baseline_y as isize) - (px * 0.32).round() as isize;

                for dy in -5..=5 {
                    for dx in -5..=5 {
                        let d2 = dx * dx + dy * dy;
                        if (16..=28).contains(&d2) {
                            let px_pos = sigil_cx + dx;
                            let py_pos = sigil_cy + dy;
                            if px_pos >= 0 && px_pos < buf_w as isize && py_pos >= 0 && py_pos < PAPER_HEIGHT as isize {
                                buffer[(py_pos as usize) * buf_w + px_pos as usize] = color;
                            }
                        }
                        if dx <= 0 && d2 <= 20 {
                            let px_pos = sigil_cx + dx;
                            let py_pos = sigil_cy + dy;
                            if px_pos >= 0 && px_pos < buf_w as isize && py_pos >= 0 && py_pos < PAPER_HEIGHT as isize {
                                buffer[(py_pos as usize) * buf_w + px_pos as usize] = color;
                            }
                        }
                    }
                }

                pen_x += sigil_w + 3.0;
                continue;
            }

            let (metrics, bitmap) = font.rasterize(ch, px);
            if metrics.width > 0 && metrics.height > 0 {
                let gx = (pen_x + metrics.xmin as f32).round() as isize;
                let gy = (baseline_y as isize) - (metrics.ymin + metrics.height as i32) as isize;

                for row in 0..metrics.height {
                    let py = gy + row as isize;
                    if py < 0 || py >= PAPER_HEIGHT as isize {
                        continue;
                    }
                    let row_offset = (py as usize) * buf_w;

                    for col in 0..metrics.width {
                        let px_pos = gx + col as isize;
                        if px_pos < 0 || px_pos >= buf_w as isize {
                            continue;
                        }

                        let raw_cov = bitmap[row * metrics.width + col] as f32 / 255.0;
                        if raw_cov <= 0.01 {
                            continue;
                        }

                        let cov = (raw_cov * 1.12).min(1.0);
                        let idx = row_offset + px_pos as usize;
                        let bg = buffer[idx];

                        let bg_r_lin = srgb_to_linear(((bg >> 16) & 0xFF) as u8);
                        let bg_g_lin = srgb_to_linear(((bg >> 8) & 0xFF) as u8);
                        let bg_b_lin = srgb_to_linear((bg & 0xFF) as u8);

                        let out_r_lin = fg_r_lin * cov + bg_r_lin * (1.0 - cov);
                        let out_g_lin = fg_g_lin * cov + bg_g_lin * (1.0 - cov);
                        let out_b_lin = fg_b_lin * cov + bg_b_lin * (1.0 - cov);

                        let nr = linear_to_srgb(out_r_lin) as u32;
                        let ng = linear_to_srgb(out_g_lin) as u32;
                        let nb = linear_to_srgb(out_b_lin) as u32;

                        buffer[idx] = (nr << 16) | (ng << 8) | nb;
                    }
                }
            }
            pen_x += metrics.advance_width;
        }
    }

    fn draw_wrapped_text_simple(
        buffer: &mut [u32],
        buf_w: usize,
        font: &Font,
        text: &str,
        x0: usize,
        y0: usize,
        max_w: f32,
        px: f32,
        line_height_mult: f32,
        color: u32,
    ) -> usize {
        let line_step = (px * line_height_mult).round() as usize;
        let mut cur_y = y0;

        for paragraph in text.lines() {
            if cur_y >= PAPER_HEIGHT - 65 {
                break;
            }

            if paragraph.trim().is_empty() {
                cur_y += line_step / 2;
                continue;
            }

            let mut current_line = String::new();
            let mut current_w = 0.0f32;
            let space_w = font.metrics(' ', px).advance_width;

            for word in paragraph.split_whitespace() {
                let mut word_w = 0.0f32;
                for ch in word.chars() {
                    match ch {
                        '◐' | '⑉' | '✕' => word_w += 17.0,
                        _ => word_w += font.metrics(ch, px).advance_width,
                    }
                }

                if current_line.is_empty() {
                    current_line.push_str(word);
                    current_w = word_w;
                } else if current_w + space_w + word_w <= max_w {
                    current_line.push(' ');
                    current_line.push_str(word);
                    current_w += space_w + word_w;
                } else {
                    Self::draw_text(buffer, buf_w, font, &current_line, x0, cur_y, px, color);
                    cur_y += line_step;
                    if cur_y >= PAPER_HEIGHT - 65 {
                        break;
                    }
                    current_line = word.to_string();
                    current_w = word_w;
                }
            }

            if !current_line.is_empty() && cur_y < PAPER_HEIGHT - 65 {
                Self::draw_text(buffer, buf_w, font, &current_line, x0, cur_y, px, color);
                cur_y += line_step;
            }
        }
        cur_y
    }
}

#[inline(always)]
fn srgb_to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
fn linear_to_srgb(v: f32) -> u8 {
    let clamped = v.clamp(0.0, 1.0);
    let s = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}