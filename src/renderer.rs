use stm32f7::lcd;

pub struct Renderer<'a> {
    display: &'a mut lcd::Lcd
}

impl<'a> Renderer<'a> {
    pub fn new(display: &'a mut lcd::Lcd) -> Self {
        Renderer{
            display: display,
        }
    }

    pub fn render_pixel(&mut self, x: u16, y: u16, color: u16) {
        self.display.print_point_color_at(x, y, color);
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