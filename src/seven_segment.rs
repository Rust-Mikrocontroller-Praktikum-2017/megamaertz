use collections::vec::Vec;
use constants;
use renderer::Renderer;

struct Segment {
    pixel: Vec<(u16, u16)>,
}

impl Segment {
    pub fn new_horizontal(x_offset: u16, y_offset: u16) -> Self {
        let mut result: Vec<(u16, u16)> = Vec::new();
        for y in 0..constants::SEGMENT_SIZE.1 {
            for x in 0..constants::SEGMENT_SIZE.0 {
                result.push(((x + x_offset) as u16, (y + y_offset) as u16));
            }
        }

        Segment { pixel: result }
    }

    pub fn new_vertical(x_offset: u16, y_offset: u16) -> Self {
        let mut result: Vec<(u16, u16)> = Vec::new();
        for y in 0..constants::SEGMENT_SIZE.0 {
            for x in 0..constants::SEGMENT_SIZE.1 {
                result.push(((x + x_offset) as u16, (y + y_offset) as u16));
            }
        }

        Segment { pixel: result }
    }
}

pub struct SSDisplay {
    segs: [Segment; 7],
    x: u16,
    y: u16,
}

impl SSDisplay {
    pub fn new(x: u16, y: u16) -> Self {
        SSDisplay {
            segs: [Segment::new_horizontal(constants::SEGMENT_SIZE.1, 0),
                   Segment::new_vertical(constants::SEGMENT_SIZE.1 + constants::SEGMENT_SIZE.0,
                                         constants::SEGMENT_SIZE.1),
                   Segment::new_vertical(constants::SEGMENT_SIZE.1 + constants::SEGMENT_SIZE.0,
                                         constants::SEGMENT_SIZE.0 * 2),
                   Segment::new_horizontal(constants::SEGMENT_SIZE.1,
                                           constants::SEGMENT_SIZE.0 * 3),
                   Segment::new_vertical(0, constants::SEGMENT_SIZE.0 * 2),
                   Segment::new_vertical(0, constants::SEGMENT_SIZE.1),
                   Segment::new_horizontal(constants::SEGMENT_SIZE.1,
                                           constants::SEGMENT_SIZE.0 + constants::SEGMENT_SIZE.1)],
            x: x,
            y: y,
        }
    }

    pub fn render(&mut self, n: u16, color: u16, rend: &mut Renderer) {
        let bcd = u16_to_bcd(n);
        let mut offset = 0;
        for i in (0..5).rev() {
            let (print, alpha) = get_segment_indices(bcd[i]);
            for s_num in print {
                let seg = &self.segs[s_num];
                for p in &seg.pixel {
                    // result.push((p.0 + offset + self.x, p.1 + self.y, color));
                    rend.render_pixel(p.0 + offset + self.x, p.1 + self.y, color);
                }
            }

            for a_num in alpha {
                let seg = &self.segs[a_num];
                for p in &seg.pixel {
                    // result.push((p.0 + offset + self.x, p.1 + self.y, 0x0000));
                    rend.render_pixel(p.0 + offset + self.x, p.1 + self.y, 0x0000);
                }
            }

            offset += constants::ELEMENT_GAP + Self::get_element_width();
        }
    }

    pub fn get_width() -> u16 {
        5 * Self::get_element_width() + 4 * constants::ELEMENT_GAP
    }

    pub fn get_height() -> u16 {
        Self::get_element_height()
    }

    pub fn get_element_width() -> u16 {
        2 * constants::SEGMENT_SIZE.1 + constants::SEGMENT_SIZE.0
    }

    pub fn get_element_height() -> u16 {
        2 * constants::SEGMENT_SIZE.0 + 3 * constants::SEGMENT_SIZE.1
    }
}

fn u16_to_bcd(n: u16) -> [u16; 5] {
    let mut tmp = n;
    let mut result: [u16; 5] = [0; 5];

    result[4] = tmp / 10000;
    tmp -= result[4] * 10000;
    result[3] = tmp / 1000;
    tmp -= result[3] * 1000;
    result[2] = tmp / 100;
    tmp -= result[2] * 100;
    result[1] = tmp / 10;
    tmp -= result[1] * 10;
    result[0] = tmp;

    result
}


fn get_segment_indices(num: u16) -> (Vec<usize>, Vec<usize>) {
    let mut print: Vec<usize> = Vec::new();
    let mut alpha: Vec<usize> = Vec::new();

    match num {
        0 => {
            push_to_vec(&mut print, &[0, 1, 2, 3, 4, 5]);
            push_to_vec(&mut alpha, &[6]);
        }
        1 => {
            push_to_vec(&mut print, &[1, 2]);
            push_to_vec(&mut alpha, &[0, 3, 4, 5, 6]);
        }
        2 => {
            push_to_vec(&mut print, &[0, 1, 3, 4, 6]);
            push_to_vec(&mut alpha, &[2, 5]);
        }
        3 => {
            push_to_vec(&mut print, &[0, 1, 2, 3, 6]);
            push_to_vec(&mut alpha, &[4, 5]);
        }
        4 => {
            push_to_vec(&mut print, &[1, 2, 5, 6]);
            push_to_vec(&mut alpha, &[0, 3, 4]);
        }
        5 => {
            push_to_vec(&mut print, &[0, 2, 3, 5, 6]);
            push_to_vec(&mut alpha, &[1, 4]);
        }
        6 => {
            push_to_vec(&mut print, &[0, 2, 3, 4, 5, 6]);
            push_to_vec(&mut alpha, &[1]);
        }
        7 => {
            push_to_vec(&mut print, &[0, 1, 2]);
            push_to_vec(&mut alpha, &[3, 4, 5, 6]);
        }
        8 => {
            push_to_vec(&mut print, &[0, 1, 2, 3, 4, 5, 6]);
        }
        9 => {
            push_to_vec(&mut print, &[0, 1, 2, 3, 5, 6]);
            push_to_vec(&mut alpha, &[4]);
        }
        _ => {
            push_to_vec(&mut print, &[6]);
            push_to_vec(&mut alpha, &[0, 1, 2, 3, 4, 5]);
        }
    }

    (print, alpha)
}

fn push_to_vec(vec: &mut Vec<usize>, slice: &[usize]) {
    for i in slice {
        vec.push(*i)
    }
}
