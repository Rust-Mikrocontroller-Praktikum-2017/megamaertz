use stm32f7::lcd;

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

    pub fn render_pixel(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x, y) {
            self.display.print_point_color_at(x, y, color);
        }
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
            self.render_pixel(x.wrapping_add(i), y, 0x0000);
            self.render_pixel(x.wrapping_sub(i), y, 0x0000);
            self.render_pixel(x, y.wrapping_add(i), 0x0000);
            self.render_pixel(x, y.wrapping_sub(i), 0x0000);
        }
    }
}