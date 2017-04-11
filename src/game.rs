use core;
use constants;
use random;
use renderer;
use collections::vec::Vec;
use stm32f7::board::sai::Sai;



pub fn vol_limit_reached(sai_2: &'static Sai) -> bool {
    while !sai_2.bsr.read().freq() {} // fifo_request_flag
    let data0 = sai_2.bdr.read().data() as i16 as i32;
    while !sai_2.bsr.read().freq() {} // fifo_request_flag
    let data1 = sai_2.bdr.read().data() as i16 as i32;

    let mic_data = if data0.abs() > data1.abs() {
        data0.abs() as u16
    } else {
        data1.abs() as u16
    };

    // mic_data reprents our "volume". Magic number 420 after testing.
    let blaze_it = 2000;
    mic_data > blaze_it
}

pub fn get_rnd_lifetime(rnd: &mut random::Rng) -> usize {
    let mut num = rnd.rand() as usize;
    num &= 0x3FFF;
    core::cmp::max(num, 5000)
}

pub fn get_rnd_pos(rand: &mut random::Rng,
                   existing_hero: &[Target],
                   existing_evil: &[Target])
                   -> (u16, u16) {
    let mut pos = renderer::Renderer::get_random_pos(rand,
                                                     constants::TARGET_SIZE_50.0,
                                                     constants::TARGET_SIZE_50.1);
    while !pos_is_okay(pos, existing_hero, existing_evil) {
        pos = renderer::Renderer::get_random_pos(rand,
                                                 constants::TARGET_SIZE_50.0,
                                                 constants::TARGET_SIZE_50.1);
    }
    pos
}

pub fn are_overlapping_targets(target: &Target, pos: (u16, u16)) -> bool {
    let corner_ul = (target.x, target.y);
    let corner_lr = (target.x + target.width, target.y + target.height);

    let x1 = pos.0;
    let y1 = pos.1;
    let x2 = pos.0 + constants::TARGET_SIZE_50.0;
    let y2 = pos.1 + constants::TARGET_SIZE_50.1;

    point_is_within((x1, y1), corner_ul, corner_lr) ||
    point_is_within((x2, y2), corner_ul, corner_lr) ||
    point_is_within((x1, y2), corner_ul, corner_lr) ||
    point_is_within((x2, y1), corner_ul, corner_lr)
}

pub fn point_is_within(point: (u16, u16), corner_ul: (u16, u16), corner_lr: (u16, u16)) -> bool {
    point.0 >= corner_ul.0 && point.0 <= corner_lr.0 && point.1 >= corner_ul.1 &&
    point.1 <= corner_lr.1
}

pub fn pos_is_okay(pos: (u16, u16), existing_hero: &[Target], existing_evil: &[Target]) -> bool {
    for hero in existing_hero {
        if are_overlapping_targets(hero, pos) {
            return false;
        }
    }
    for evil in existing_evil {
        if are_overlapping_targets(evil, pos) {
            return false;
        }
    }
    true
}

pub struct Target {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub bounty: u16,
    pub birthday: usize,
    pub lifetime: usize,
}

impl Target {
    pub fn new(x: u16,
               y: u16,
               width: u16,
               height: u16,
               bounty: u16,
               birthday: usize,
               lifetime: usize)
               -> Self {
        Target {
            x: x,
            y: y,
            width: width,
            height: height,
            bounty: bounty,
            birthday: birthday,
            lifetime: lifetime,
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

