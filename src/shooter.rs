use collections::vec::Vec;

// const TARGET_WIDTH: u16 = 50;
// const TARGET_HEIGHT: u16 = 50;

pub struct Target {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub bounty: u16,
    pub birthday: usize
}

impl Target {
    pub fn new(x: u16, y: u16, width: u16, height: u16, bounty: u16, birthday: usize) -> Self {
        Target {
            x: x,
            y: y,
            width: width,
            height: height,
            bounty: bounty,
            birthday: birthday
        }
    }

    fn coord_is_inside(&mut self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn check_for_hit(targets: &mut [Target], touches: &[(u16, u16)]) -> Vec<usize> {
        let mut indices: Vec<usize> = Vec::new();
        for (i, target) in targets.iter_mut().enumerate() {
            for touch in touches {
                if target.coord_is_inside(touch.0, touch.1) {
                    indices.push(i);
                }
            }
        }
        indices
    }
}
