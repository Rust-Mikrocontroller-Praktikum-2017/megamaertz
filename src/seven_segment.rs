use collections::vec::Vec;
use renderer::Renderer;

struct Segment {
    pixel: Vec<(u16, u16)>,
}

impl Segment {
    pub fn new_horizontal(offset: (u16, u16), seg_size: (u16, u16)) -> Self {
        let mut result: Vec<(u16, u16)> = Vec::new();
        for y in 0..seg_size.1 {
            for x in 0..seg_size.0 {
                result.push((x + offset.0, y + offset.1));
            }
        }

        Segment { pixel: result }
    }

    pub fn new_vertical(offset: (u16, u16), seg_size: (u16, u16)) -> Self {
        let mut result: Vec<(u16, u16)> = Vec::new();
        for y in 0..seg_size.0 {
            for x in 0..seg_size.1 {
                result.push((x + offset.0, y + offset.1));
            }
        }

        Segment { pixel: result }
    }
}

pub struct SSDisplay {
    segs: [Segment; 7],
    pos: (u16, u16),
    elem_width: u16,
    gap: u16,
}

impl SSDisplay {
    // scales elemts in a factor of 2/1 (height/width) with the given width
    pub fn new(pos: (u16, u16), elem_width: u16, gap: u16) -> Self {
        let seg_size = (elem_width / 2, elem_width / 4);
        SSDisplay {
            segs: [Segment::new_horizontal((seg_size.1, 0), seg_size),
                   Segment::new_vertical((seg_size.1 + seg_size.0, seg_size.1), seg_size),
                   Segment::new_vertical((seg_size.1 + seg_size.0, seg_size.0 * 2), seg_size),
                   Segment::new_horizontal((seg_size.1, seg_size.0 * 3), seg_size),
                   Segment::new_vertical((0, seg_size.0 * 2), seg_size),
                   Segment::new_vertical((0, seg_size.1), seg_size),
                   Segment::new_horizontal((seg_size.1, seg_size.0 + seg_size.1), seg_size)],
            pos: pos,
            elem_width: elem_width,
            gap: gap,
        }
    }

    pub fn render(&self, n: u16, color: u16, rend: &mut Renderer) {
        let bcd = u16_to_bcd(n);
        let mut offset = 0;
        for i in (0..5).rev() {
            let (print, alpha) = get_segment_indices(bcd[i]);
            for s_num in print {
                let seg = &self.segs[s_num];
                for p in &seg.pixel {
                    // result.push((p.0 + offset + self.pos.0, p.1 + self.pos.1, color));
                    rend.render_pixel(p.0 + offset + self.pos.0, p.1 + self.pos.1, color);
                }
            }

            for a_num in alpha {
                let seg = &self.segs[a_num];
                for p in &seg.pixel {
                    // result.push((p.0 + offset + self.pos.0, p.1 + self.pos.1, 0x0000));
                    rend.render_pixel(p.0 + offset + self.pos.0, p.1 + self.pos.1, 0x0000);
                }
            }

            offset += self.elem_width + self.gap;
        }
    }

    pub fn calculate_width(elem_width: u16, gap: u16) -> u16 {
        5 * elem_width + 4 * gap
    }

    pub fn calculate_height(elem_width: u16) -> u16 {
        2 * elem_width
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
