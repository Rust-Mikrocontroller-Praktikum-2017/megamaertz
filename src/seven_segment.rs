use collections::vec::Vec;
use renderer::Renderer;
use constants;

struct Segment {
    pixel: Vec<(u16, u16)>,
}

impl Segment {
    pub fn new(offset: (u16, u16), size: (u16, u16)) -> Self {
        let mut result: Vec<(u16, u16)> = Vec::new();
        for x in 0..size.0 {
            for y in 0..size.1 {
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
        let seg_size = (elem_width / constants::HEIGHT_TO_WIDTH_SCALING,
                        elem_width / (constants::HEIGHT_TO_WIDTH_SCALING * 2));
        SSDisplay {
            segs: [// real seven segements
                   Segment::new((seg_size.1, 0), seg_size),
                   Segment::new((seg_size.1 + seg_size.0, seg_size.1), flip(seg_size)),
                   Segment::new((seg_size.1 + seg_size.0, seg_size.0 * 2), flip(seg_size)),
                   Segment::new((seg_size.1, seg_size.0 * 3), seg_size),
                   Segment::new((0, seg_size.0 * 2), flip(seg_size)),
                   Segment::new((0, seg_size.1), flip(seg_size)),
                   Segment::new((seg_size.1, seg_size.0 + seg_size.1), seg_size),
                   ],
            pos: pos,
            elem_width: elem_width,
            gap: gap,
        }
    }

    pub fn render(&self, n: u16, color: u16, rend: &mut Renderer) {
        self.render_offset(n, color, 0, rend);
    }

    fn render_offset(&self, n: u16, color: u16, offset: u16, rend: &mut Renderer) {
        let bcd = u16_to_bcd(n);
        let mut offs = offset;
        for i in (0..5).rev() {
            let (print, alpha) = get_segment_indices(bcd[i]);
            self.render_segments(&print, color, offs, rend);
            self.render_segments(&alpha, 0x0000, offs, rend);

            offs += self.elem_width + self.gap;
        }
    }

    pub fn render_hs(&self, n: u16, color: u16, rend: &mut Renderer) {
        let h = ([1, 2, 4, 5, 6], [0, 3]);
        let s = ([0, 2, 3, 5, 6], [1, 4]);
        let minus = ([6], [0, 1, 2, 3, 4, 5]);

        let mut offset = 0;
        self.render_segments(&h.0, color, offset, rend);
        self.render_segments(&h.1, 0x0000, offset, rend);

        offset += self.elem_width + self.gap;
        self.render_segments(&s.0, color, offset, rend);
        self.render_segments(&s.1, 0x0000, offset, rend);

        offset += self.elem_width + constants::HS_SPACE_SIZE;
        self.render_segments(&minus.0, color, offset, rend);
        self.render_segments(&minus.1, 0x0000, offset, rend);

        offset += self.elem_width + constants::HS_SPACE_SIZE;
        self.render_offset(n, color, offset, rend);
    }

    fn render_segments(&self, segs: &[usize], color: u16, offset: u16, rend: &mut Renderer) {
        for seg_num in segs {
            let seg = &self.segs[*seg_num];
            for p in &seg.pixel {
                rend.render_pixel(p.0 + offset + self.pos.0, p.1 + self.pos.1, color);
            }
        }
    }

    pub fn calculate_width(elem_width: u16, gap: u16) -> u16 {
        5 * elem_width + 4 * gap
    }

    pub fn calculate_height(elem_width: u16) -> u16 {
        2 * elem_width
    }

    pub fn calculate_hs_prefix_width(elem_width: u16, gap: u16) -> u16 {
        3 * elem_width + gap + 2 * constants::HS_SPACE_SIZE
    }

    pub fn calculate_hs_width(elem_width: u16, gap: u16) -> u16 {
        Self::calculate_hs_prefix_width(elem_width, gap) + Self::calculate_width(elem_width, gap)
    }
}

fn flip(tuple: (u16, u16)) -> (u16, u16){
    (tuple.1, tuple.0)
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
