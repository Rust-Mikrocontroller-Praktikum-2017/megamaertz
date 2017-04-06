const SEGMENT_WIDTH: usize = 6;
const SEGMENT_HEIGHT: usize = 3;
const ELEMENT_COUNT: usize = 5;

const SEGMENT_COLOR: u16 = 0x0000;

struct Segment {
    pixel: [(u16, u16); SEGMENT_HEIGHT * SEGMENT_WIDTH],
}

impl Segment {
    pub fn new_horizontal(x_offset: usize, y_offset: usize) -> Self {
        let mut tmp = [(0, 0); SEGMENT_HEIGHT * SEGMENT_WIDTH];
        for y in 0..SEGMENT_HEIGHT {
            for x in 0..SEGMENT_WIDTH {
                tmp[y * SEGMENT_HEIGHT + x] = ((x + x_offset) as u16, (y + y_offset) as u16);
            }
        }

        Segment { pixel: tmp }
    }

    pub fn new_vertical(x_offset: usize, y_offset: usize) -> Self {
        let mut tmp = [(0, 0); SEGMENT_HEIGHT * SEGMENT_WIDTH];
        for y in 0..SEGMENT_WIDTH {
            for x in 0..SEGMENT_HEIGHT {
                tmp[y * SEGMENT_WIDTH + x] = ((x + x_offset) as u16, (y + y_offset) as u16);
            }
        }

        Segment { pixel: tmp }
    }
}

static a: Segment = Segment::new_horizontal(SEGMENT_HEIGHT, 0);
static b: Segment = Segment::new_vertical(SEGMENT_HEIGHT + SEGMENT_WIDTH, SEGMENT_HEIGHT);
static c: Segment = Segment::new_vertical(SEGMENT_HEIGHT + SEGMENT_WIDTH, SEGMENT_WIDTH * 2);
static d: Segment = Segment::new_horizontal(SEGMENT_HEIGHT, SEGMENT_WIDTH * 3);
static e: Segment = Segment::new_vertical(0, SEGMENT_WIDTH * 2);
static f: Segment = Segment::new_vertical(0, SEGMENT_HEIGHT);
static g: Segment = Segment::new_horizontal(SEGMENT_HEIGHT, SEGMENT_WIDTH + SEGMENT_HEIGHT);

struct Element {
    segments: [Segment; 7],
}

struct SSDisplay {
    elements: [Element; ELEMENT_COUNT],
    color: u16,
}

// impl SSDisplay {
//     pub fn new() -> Self {
//         SSDisplay {}
//     }
// }