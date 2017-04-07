
// const TARGET_WIDTH: u16 = 50;
// const TARGET_HEIGHT: u16 = 50;

pub struct Target {
    x: u16,
    y: u16,
    width: u16,
    height: u16
}

impl Target {
    pub fn new(x: u16, y: u16, width: u16, height: u18) -> Self {
        Target {
            x: x,
            y: y,
            width: width,
            height: height 
        }
    }

    fn coord_is_inside(&mut self, x: u16, y: u16) -> bool {
        x > self.width && x < self.x + self.width && y > self.y && y < self.y + self.height
    }
}