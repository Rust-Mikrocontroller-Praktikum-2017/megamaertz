use stm32f7::lcd;
use core::ptr;

const DISPLAY_WIDTH: u16 = 480;
const DISPLAY_HEIGHT: u16 = 272;


pub struct Renderer<'a> {
    display: &'a mut lcd::Lcd,
    last_touch: (u16, u16),
}

impl<'a> Renderer<'a> {
    pub fn new(display: &'a mut lcd::Lcd) -> Self {
        Renderer {
            display: display,
            last_touch: (240, 136),
        }
    }

    fn coord_is_inside(x: u16, y: u16) -> bool {
        x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT
    }

    fn render_pixel(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x, y) {
            self.display.print_point_color_at(x, y, color);
        }
    }

    fn render_bg(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x, y) {
            let addr: u32 = 0xC000_0000;
            let pixel = (y as u32) * 480 + (x as u32);
            let pixel_color = (addr + pixel * 2) as *mut u16;
            unsafe { ptr::write_volatile(pixel_color, color) };
        }
    }

    fn set_pixel_invisible(&mut self, x: u16, y: u16) {
        self.render_pixel(x, y, 0x0000);
    }

    pub fn cursor(&mut self, x: u16, y: u16) {
        self.remove_last_cursor();
        for i in 0..13 {
            self.render_pixel(x.wrapping_add(i), y, 0xFFFF);
            self.render_pixel(x.wrapping_sub(i), y, 0xFFFF);
            self.render_pixel(x, y.wrapping_add(i), 0xFFFF);
            self.render_pixel(x, y.wrapping_sub(i), 0xFFFF);
        }
        self.last_touch = (x, y);
    }

    fn remove_last_cursor(&mut self) {
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

    pub fn draw_bg_unicolor(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) {
        for dsp_y in y..y + height {
            for dsp_x in x..x + width {
                self.render_bg(dsp_x, dsp_y, color);
            }
        }
    }
}

