use collections::vec::Vec;

// const TARGET_WIDTH: u16 = 50;
// const TARGET_HEIGHT: u16 = 50;

pub struct Target {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Target {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Target {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }

    fn coord_is_inside(&mut self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn check_for_hit(targets: &mut [Target], touches: &[(u16, u16)]) -> Vec<usize> {
        let mut indices: Vec<usize> = Vec::new();
        for i in 0..targets.len() {
            for j in 0..touches.len() {
                let (x,y) = touches[j];
                if targets[i].coord_is_inside(x, y) {
                    indices.push(i);
                } 
            }
        }
        indices
    }
}

