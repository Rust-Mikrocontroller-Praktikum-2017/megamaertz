use stm32f7::lcd;
use core::ptr;
use core::cmp;
use constants;
use random;

pub struct Renderer {
    display: lcd::Lcd,
    last_touch: (u16, u16),
}

impl Renderer {
    pub fn new(display: lcd::Lcd) -> Self {
        Renderer {
            display: display,
            last_touch: (240, 136),
        }
    }

    fn coord_is_inside(x: u16, y: u16) -> bool {
        x < constants::DISPLAY_SIZE.0 && y < constants::DISPLAY_SIZE.1
    }

    pub fn render_bg(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x, y) {
            let addr: u32 = 0xC000_0000;
            let pixel = (y as u32) * 480 + (x as u32);
            let pixel_color = (addr + pixel * 2) as *mut u16;
            unsafe { ptr::write_volatile(pixel_color, color) };
        }
    }

    pub fn render_pixel(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x, y) {
            self.display.print_point_color_at(x, y, color);
        }
    }

    fn set_pixel_invisible(&mut self, x: u16, y: u16) {
        self.render_pixel(x, y, 0x0000);
    }

    pub fn cursor(&mut self, x: u16, y: u16) {
        self.remove_last_cursor();
        let c = RGBColor::from_rgb(0, 0, 0);
        for i in 0..13 {
            self.render_pixel(x.wrapping_add(i), y, c);
            self.render_pixel(x.wrapping_sub(i), y, c);
            self.render_pixel(x, y.wrapping_add(i), c);
            self.render_pixel(x, y.wrapping_sub(i), c);
        }
        self.last_touch = (x, y);
    }

    pub fn remove_last_cursor(&mut self) {
        let x = self.last_touch.0;
        let y = self.last_touch.1;
        for i in 0..13 {
            self.set_pixel_invisible(x.wrapping_add(i), y);
            self.set_pixel_invisible(x.wrapping_sub(i), y);
            self.set_pixel_invisible(x, y.wrapping_add(i));
            self.set_pixel_invisible(x, y.wrapping_sub(i));
        }
    }

    pub fn draw(&mut self, x: u16, y: u16, width: u16, img: &[u8]) {
        for i in 0..(img.len() / 2) {
            let img_idx = i * 2;
            let dsp_y = y + (i as u16 / width);
            let dsp_x = x + (i as u16 % width);
            let p = ((img[img_idx] as u16) << 8) | (img[img_idx + 1] as u16);
            self.render_pixel(dsp_x, dsp_y, p);
        }
    }

    pub fn draw_u16(&mut self, x: u16, y: u16, width: u16, img: &[u16]) {
        for (i, px) in img.iter().enumerate() {
            let dsp_y = y + (i as u16 / width);
            let dsp_x = x + (i as u16 % width);
            self.render_pixel(dsp_x, dsp_y, *px);
        }
    }

    pub fn draw_u16_tuple(&mut self, img: &[(u16, u16, u16)]) {
        for px in img.iter() {
            self.render_pixel(px.0, px.1, px.2);
        }
    }

    pub fn draw_full_bg_unicolor(&mut self, color: u16) {
        for y in 0..272 {
            for x in 0..480 {
                self.render_bg(x, y, color);
            }
        }
    }

    pub fn draw_bg(&mut self, x: u16, y: u16, width: u16, img: &[u8]) {
        for i in 0..(img.len() / 2) {
            let img_idx = i * 2;
            let dsp_y = y + (i as u16 / width);
            let dsp_x = x + (i as u16 % width);
            let p = ((img[img_idx] as u16) << 8) | (img[img_idx + 1] as u16);
            self.render_bg(dsp_x, dsp_y, p);
        }
    }

    pub fn draw_bg_u16(&mut self, x: u16, y: u16, width: u16, img: &[u16]) {
        for (i, px) in img.iter().enumerate() {
            let dsp_y = y + (i as u16 / width);
            let dsp_x = x + (i as u16 % width);
            self.render_bg(dsp_x, dsp_y, *px);
        }
    }

    pub fn draw_bg_unicolor(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) {
        for dsp_y in y..y + height {
            for dsp_x in x..x + width {
                self.render_bg(dsp_x, dsp_y, color);
            }
        }
    }

    pub fn draw_dump(&mut self, x: u16, y: u16, size: (u16, u16), dump: &[u8]) {
        use renderer::RGBColor;

        let img_cnt = size.0 as usize * size.1 as usize;
        for i in 0..img_cnt {
            let idx = i * 4;
            let dsp_y = y + (i / size.0 as usize) as u16;
            let dsp_x = x + (i % size.0 as usize) as u16;
            let c = RGBColor::from_rgb_with_alpha(dump[idx + 3],
                                                  dump[idx],
                                                  dump[idx + 1],
                                                  dump[idx + 2]);
            self.render_pixel(dsp_x, dsp_y, c);
        }
    }

    pub fn clear(&mut self, x: u16, y: u16, size: (u16, u16)) {
        let img_cnt = size.0 as usize * size.1 as usize;
        for i in 0..img_cnt {
            let dsp_y = y + (i / size.0 as usize) as u16;
            let dsp_x = x + (i % size.0 as usize) as u16;
            self.set_pixel_invisible(dsp_x, dsp_y);
        }
    }

    pub fn draw_dump_bg(&mut self, x: u16, y: u16, size: (u16, u16), dump: &[u8]) {
        use renderer::RGBColor;

        let img_cnt = size.0 as usize * size.1 as usize;
        for i in 0..img_cnt {
            let idx = i * 4;
            let dsp_y = y + (i / size.0 as usize) as u16;
            let dsp_x = x + (i % size.0 as usize) as u16;
            let c = RGBColor::from_rgb_with_alpha(dump[idx + 3],
                                                  dump[idx],
                                                  dump[idx + 1],
                                                  dump[idx + 2]);
            self.render_bg(dsp_x, dsp_y, c)
        }
    }

    pub fn get_random_pos(rng: &mut random::MTRng32, width: u16, height: u16) -> (u16, u16) {
        (cmp::min(rng.rand() as u16 % constants::DISPLAY_SIZE.0,
                  constants::DISPLAY_SIZE.0 - width - 1),
         cmp::min(rng.rand() as u16 % constants::DISPLAY_SIZE.1,
                  constants::DISPLAY_SIZE.1 - height - 1))
    }
}

pub struct RGBColor();

impl RGBColor {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> u16 {
        Self::from_rgb_with_alpha(42, r, g, b)
    }

    pub fn from_rgb_with_alpha(a: u8, r: u8, g: u8, b: u8) -> u16 {
        let r_f = (r / 8) as u16;
        let g_f = (g / 8) as u16;
        let b_f = (b / 8) as u16;
        let c: u16 = if a >= 42 { 1 << 15 } else { 0 };
        c | (r_f << 10) | (g_f << 5) | b_f
    }

    pub fn from_hex_with_alpha(color: u32) -> u16 {
        let a = (color >> 24) as u8;
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        Self::from_rgb_with_alpha(a, r, g, b)
    }

    pub fn from_hex(color: u32) -> u16 {
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        Self::from_rgb(r, g, b)
    }
}
