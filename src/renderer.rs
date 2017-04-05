use stm32f7::lcd;

const DISPLAY_WIDTH: u16 = 480;
const DISPLAY_HEIGHT: u16 = 272;


pub struct Renderer<'a> {
    display: &'a mut lcd::Lcd
}

impl<'a> Renderer<'a> {
    pub fn new(display: &'a mut lcd::Lcd) -> Self {
        Renderer{
            display: display,
        }
    }

    fn coord_is_inside(x: u16, y: u16) -> bool {
        x > 0 && x < DISPLAY_HEIGHT && y > 0 && y < DISPLAY_WIDTH
    }

    pub fn render_pixel(&mut self, x: u16, y: u16, color: u16) {
        if Self::coord_is_inside(x,y) {
            self.display.print_point_color_at(x, y, color);
        };
    }

    pub fn cursor(&mut self, x:u16, y:u16) {
        for i in 0..13 {
            self.render_pixel(x + i, y, 0xFFFF);
            self.render_pixel(x - i, y, 0xFFFF);
            self.render_pixel(x, y + i, 0xFFFF);
            self.render_pixel(x, y - i, 0xFFFF);
        }

    }

    pub fn draw_colorful_square(&mut self) {
        let colors = [0xFF00, 0xAF00];
        let start = (240, 136);
        let border_width = 4;

        for n in 0..20 {
            let color = colors[(n as usize) % 2];
            let radius = n * border_width;
            for j in 0..border_width {
                for i in 0..radius + 1 + j {
                    // top and bottom
                    self.render_pixel(start.0 - i, start.1 - radius - j, color);
                    self.render_pixel(start.0 + i, start.1 - radius - j, color);
                    self.render_pixel(start.0 - i, start.1 + radius + j, color);
                    self.render_pixel(start.0 + i, start.1 + radius + j, color);

                    // left and right
                    self.render_pixel(start.0 - radius - j, start.1 - i, color);
                    self.render_pixel(start.0 - radius - j, start.1 + i, color);
                    self.render_pixel(start.0 + radius + j, start.1 - i, color);
                    self.render_pixel(start.0 + radius + j, start.1 + i, color);
                }
            }
        }
    }
}