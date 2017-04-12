use constants;
use random;
use renderer;
use collections::vec::Vec;
use seven_segment::SSDisplay;
use stm32f7::board::sai::Sai;
use stm32f7::system_clock;

pub struct Game {
    evil_targets: Vec<Target>,
    hero_targets: Vec<Target>,
    rend: renderer::Renderer,
    score: u16,
    highscore: u16,
    countdown: u16,
    rand: random::MTRng32,
    tick: usize,
    last_super_target_render_time: usize,
    super_target_hiding_duration: usize,
    last_ssd_render_time: usize,
    ss_ctr_display: SSDisplay,
    ss_hs_display: SSDisplay,
    hero_target_img: &'static [u8],
    super_target_img: &'static [u8],
    evil_target_img: &'static [u8],
    silent_mode: bool,
}

impl Game {
    pub fn new(rend: renderer::Renderer, rand: random::MTRng32) -> Self {
        let tick = system_clock::ticks();
        Game {
            evil_targets: Vec::new(),
            hero_targets: Vec::new(),
            rend: rend,
            score: 0,
            highscore: 0,
            countdown: constants::GAME_TIME,
            rand: rand,
            tick: tick,
            last_super_target_render_time: tick,
            super_target_hiding_duration: 0,
            last_ssd_render_time: tick,
            ss_ctr_display:
                SSDisplay::new((constants::DISPLAY_SIZE.0 -
                                SSDisplay::calculate_width(constants::ELEMENT_WIDTH_SMALL,
                                                           constants::ELEMENT_GAP_SMALL),
                                0),
                               constants::ELEMENT_WIDTH_SMALL,
                               constants::ELEMENT_GAP_SMALL),
            ss_hs_display: SSDisplay::new((0, 0),
                                          constants::ELEMENT_WIDTH_SMALL,
                                          constants::ELEMENT_GAP_SMALL),
            hero_target_img: constants::MEXICAN,
            super_target_img: constants::SUPER_TRUMP,
            evil_target_img: constants::TRUMP,
            silent_mode: false,
        }
    }

    pub fn draw_game_banner(&mut self) {
        let start_y = constants::GAME_OVER_OFFSET_Y - 20;
        self.rend
            .draw_dump(0, start_y, constants::FIESTA_SIZE, constants::FIESTA_PINATA);
    }

    pub fn draw_start_banner(&mut self) {
        self.clear_banner();
        // draw start banner
        let start_y = constants::GAME_OVER_OFFSET_Y - 20;
        self.rend
            .draw_dump(0, start_y, constants::START_SIZE, constants::START);
        //draw game mode banner
        let offset_x = constants::DISPLAY_SIZE.0 / 4 - constants::GAME_MODE_BTN_SIZE.0 / 2;
        self.rend
            .draw_dump(offset_x,
                       start_y + constants::START_SIZE.1 + 20,
                       constants::GAME_MODE_BTN_SIZE,
                       constants::BURGER_MODE);
        self.rend
            .draw_dump(offset_x + constants::DISPLAY_SIZE.0 / 2,
                       start_y + constants::START_SIZE.1 + 20,
                       constants::GAME_MODE_BTN_SIZE,
                       constants::TACO_MODE);
    }

    pub fn start(&mut self, touch: (u16, u16)) {
        self.ss_ctr_display
            .render(constants::GAME_TIME, constants::BLACK, &mut self.rend);
        self.ss_hs_display
            .render(0, constants::BLACK, &mut self.rend);
        let tick = system_clock::ticks();
        self.last_ssd_render_time = tick;
        self.last_super_target_render_time = tick;
        self.super_target_hiding_duration =
            Self::get_rnd_lifetime(&mut self.rand,
                                   constants::SUPER_TARGET_HIDING_DURATION.0,
                                   constants::SUPER_TARGET_HIDING_DURATION.1);
        self.countdown = constants::GAME_TIME;
        self.clear_banner();

        // which game mode?
        if touch.0 > constants::DISPLAY_SIZE.0 / 2 {
            self.hero_target_img = constants::MEXICAN;
            self.super_target_img = constants::SUPER_TRUMP;
            self.evil_target_img = constants::TRUMP;
        } else {
            self.hero_target_img = constants::TRUMP;
            self.super_target_img = constants::SUPER_MEXICAN;
            self.evil_target_img = constants::MEXICAN;
        }
        self.draw_silent_button();
    }

    fn draw_silent_button(&mut self) {
        let silent_btn = if self.silent_mode {
            constants::SILENT_BTN
        } else {
            constants::SILENT_BTN_NEG
        };
        self.rend
            .draw_dump(0,
                       constants::DISPLAY_SIZE.1 - constants::SILENT_BTN_SIZE.1,
                       constants::SILENT_BTN_SIZE,
                       silent_btn);
    }

    fn clear_banner(&mut self) {
        let offset_y = constants::GAME_OVER_OFFSET_Y - 20;
        let h = constants::DISPLAY_SIZE.1 - offset_y;
        self.rend
            .clear(0, offset_y, (constants::DISPLAY_SIZE.0, h));
    }

    pub fn update_countdown(&mut self) -> u16 {
        self.tick = system_clock::ticks();
        if self.tick - self.last_ssd_render_time >= 1000 {
            self.countdown -= if self.countdown > 0 { 1 } else { 0 };
            let color = if self.countdown <= 5 {
                constants::RED
            } else {
                constants::BLACK
            };
            self.ss_ctr_display
                .render(self.countdown, color, &mut self.rend);
            self.last_ssd_render_time = self.tick;
        }
        self.countdown
    }

    pub fn draw_missing_targets(&mut self) {
        // rendering random positioned evil targets
        while self.evil_targets.len() < constants::MAX_EVIL_TARGETS {
            let lifetime = Self::get_rnd_lifetime(&mut self.rand, 3000, 5000);
            let pos: (u16, u16) =
                Self::get_rnd_pos(&mut self.rand, &self.hero_targets, &self.evil_targets);
            let evil_target = Target::new(pos.0,
                                          pos.1,
                                          constants::TARGET_SIZE.0,
                                          constants::TARGET_SIZE.1,
                                          constants::EVIL_POINTS,
                                          self.tick,
                                          lifetime);
            let super_evil_target = Target::new(pos.0,
                                                pos.1,
                                                constants::TARGET_SIZE.0,
                                                constants::TARGET_SIZE.1,
                                                constants::SUPER_EVIL_POINTS,
                                                self.tick,
                                                2000);
            if self.tick - self.last_super_target_render_time >= self.super_target_hiding_duration {
                self.rend
                    .draw_dump(pos.0, pos.1, constants::TARGET_SIZE, self.super_target_img);
                self.last_super_target_render_time = self.tick;
                self.evil_targets.push(super_evil_target);
                self.super_target_hiding_duration =
                    Self::get_rnd_lifetime(&mut self.rand,
                                           constants::SUPER_TARGET_HIDING_DURATION.0,
                                           constants::SUPER_TARGET_HIDING_DURATION.1);
            } else {
                self.rend
                    .draw_dump(pos.0, pos.1, constants::TARGET_SIZE, self.evil_target_img);
                self.evil_targets.push(evil_target);
            }
        }

        // rendering random positioned hero targets
        while self.hero_targets.len() < constants::MAX_HERO_TARGETS {
            let lifetime = Self::get_rnd_lifetime(&mut self.rand, 3000, 5000);
            let pos: (u16, u16) =
                Self::get_rnd_pos(&mut self.rand, &self.hero_targets, &self.evil_targets);
            let hero_target = Target::new(pos.0,
                                          pos.1,
                                          constants::TARGET_SIZE.0,
                                          constants::TARGET_SIZE.1,
                                          constants::HERO_POINTS,
                                          self.tick,
                                          lifetime);
            self.rend
                .draw_dump(pos.0, pos.1, constants::TARGET_SIZE, self.hero_target_img);
            self.hero_targets.push(hero_target);
        }
    }

    pub fn process_shooting(&mut self, sai_2: &'static Sai, touches: Vec<(u16, u16)>) {
        if Self::silent_btn_touched(&touches) {
            self.silent_mode = !self.silent_mode;
            self.draw_silent_button();
            ::stm32f7::system_clock::wait(250);
        }
        if !self.silent_mode && !Self::vol_limit_reached(sai_2) {
            return;
        }
        let mut hit_evil_targets = Target::check_for_hit(&mut self.evil_targets, &touches);
        hit_evil_targets.sort();
        for hit_index in hit_evil_targets.iter().rev() {
            let t = self.evil_targets.remove(*hit_index);
            self.rend.clear(t.x, t.y, (t.width, t.height));
            self.score += t.bounty;
            self.ss_hs_display
                .render(self.score, constants::GREEN, &mut self.rend);
        }
        let mut hit_hero_targets = Target::check_for_hit(&mut self.hero_targets, &touches);
        hit_hero_targets.sort();
        for hit_index in hit_hero_targets.iter().rev() {
            let t = self.hero_targets.remove(*hit_index);
            self.rend.clear(t.x, t.y, (t.width, t.height));
            self.score -= if self.score < t.bounty {
                self.score
            } else {
                t.bounty
            };
            self.ss_hs_display
                .render(self.score, constants::RED, &mut self.rend);
        }
    }

    pub fn purge_old_targets(&mut self) {
        let mut targets = [&mut self.evil_targets, &mut self.hero_targets];

        // dont let targets live longer than lifetime secs
        for target_vec in &mut targets {
            for i in (0..target_vec.len()).rev() {
                if self.tick - target_vec[i].birthday > target_vec[i].lifetime {
                    let t = target_vec.remove(i);
                    self.rend.clear(t.x, t.y, (t.width, t.height));
                }
            }
        }
    }

    pub fn reset_game(&mut self) {
        for t in &self.evil_targets {
            self.rend.clear(t.x, t.y, (t.width, t.height));
        }

        for t in &self.hero_targets {
            self.rend.clear(t.x, t.y, (t.width, t.height));
        }

        self.evil_targets = Vec::new();
        self.hero_targets = Vec::new();
        self.countdown = 0;
        self.score = 0;
    }

    pub fn game_over(&mut self) {
        if self.score > self.highscore {
            self.highscore = self.score
        }

        let score = self.score;
        self.reset_game();
        self.rend
            .clear(0,
                   constants::DISPLAY_SIZE.1 - constants::SILENT_BTN_SIZE.1 - 1,
                   constants::SILENT_BTN_SIZE);
        self.rend
            .draw_dump(0,
                       constants::GAME_OVER_OFFSET_Y,
                       constants::GAME_OVER_SIZE,
                       constants::GAMEOVER);

        let ss_width = SSDisplay::calculate_width(constants::ELEMENT_WIDTH_BIG,
                                                  constants::ELEMENT_GAP_BIG);
        let ss_x = (constants::DISPLAY_SIZE.0 - ss_width) / 2;
        let ss_y = constants::GAME_OVER_OFFSET_Y + constants::GAME_OVER_SIZE.1 + 6;
        let ss_end_display = SSDisplay::new((ss_x, ss_y),
                                            constants::ELEMENT_WIDTH_BIG,
                                            constants::ELEMENT_GAP_BIG);
        ss_end_display.render(score, constants::BLACK, &mut self.rend);

        let hs_x = ((constants::DISPLAY_SIZE.0 - ss_width) / 2) -
                   SSDisplay::calculate_hs_prefix_width(constants::ELEMENT_WIDTH_BIG,
                                                        constants::ELEMENT_GAP_BIG);
        let hs_y = ss_y + SSDisplay::calculate_height(constants::ELEMENT_WIDTH_BIG) + 6;
        let hs_end_display = SSDisplay::new((hs_x, hs_y),
                                            constants::ELEMENT_WIDTH_BIG,
                                            constants::ELEMENT_GAP_BIG);
        hs_end_display.render_hs(self.highscore, constants::BLACK, &mut self.rend);
    }


    fn vol_limit_reached(sai_2: &'static Sai) -> bool {
        while !sai_2.bsr.read().freq() {} // fifo_request_flag
        let data0 = sai_2.bdr.read().data() as i16 as i32;
        while !sai_2.bsr.read().freq() {} // fifo_request_flag
        let data1 = sai_2.bdr.read().data() as i16 as i32;

        let mic_data = if data0.abs() > data1.abs() {
            data0.abs() as u16
        } else {
            data1.abs() as u16
        };

        // mic_data reprents our "volume". Magic number after testing.
        mic_data > constants::VOLUME_THRESHOLD
    }

    fn silent_btn_touched(touches: &[(u16, u16)]) -> bool {
        let silent_ul = (0, constants::DISPLAY_SIZE.1 - constants::SILENT_BTN_SIZE.1);
        let silent_lr = (constants::SILENT_BTN_SIZE.0, constants::DISPLAY_SIZE.1);
        for touch in touches {
            if Self::point_is_within((touch.0, touch.1), silent_ul, silent_lr) {
                return true;
            }
        }
        false
    }

    fn get_rnd_lifetime(rnd: &mut random::MTRng32, min: usize, max: usize) -> usize {
        let range = max - min;
        min + ((rnd.rand() as usize) % range)
    }

    fn get_rnd_pos(rand: &mut random::MTRng32,
                   existing_hero: &[Target],
                   existing_evil: &[Target])
                   -> (u16, u16) {
        let mut pos = renderer::Renderer::get_random_pos(rand,
                                                         constants::TARGET_SIZE.0,
                                                         constants::TARGET_SIZE.1);
        while !Self::pos_is_okay(pos, existing_hero, existing_evil) {
            pos = renderer::Renderer::get_random_pos(rand,
                                                     constants::TARGET_SIZE.0,
                                                     constants::TARGET_SIZE.1);
        }
        pos
    }

    fn are_overlapping_targets(target: &Target, pos: (u16, u16)) -> bool {
        let corner_ul = (target.x, target.y);
        let corner_lr = (target.x + target.width, target.y + target.height);

        let x1 = pos.0;
        let y1 = pos.1;
        let x2 = pos.0 + constants::TARGET_SIZE.0;
        let y2 = pos.1 + constants::TARGET_SIZE.1;

        Self::point_is_within((x1, y1), corner_ul, corner_lr) ||
        Self::point_is_within((x2, y2), corner_ul, corner_lr) ||
        Self::point_is_within((x1, y2), corner_ul, corner_lr) ||
        Self::point_is_within((x2, y1), corner_ul, corner_lr)
    }

    fn point_is_within(point: (u16, u16), corner_ul: (u16, u16), corner_lr: (u16, u16)) -> bool {
        point.0 >= corner_ul.0 && point.0 <= corner_lr.0 && point.1 >= corner_ul.1 &&
        point.1 <= corner_lr.1
    }

    fn pos_is_okay(pos: (u16, u16), existing_hero: &[Target], existing_evil: &[Target]) -> bool {
        // check score, timer and silent button
        let score_ul = (0, 0);
        let score_lr = (SSDisplay::calculate_width(constants::ELEMENT_WIDTH_SMALL,
                                                   constants::ELEMENT_GAP_SMALL),
                        SSDisplay::calculate_height(constants::ELEMENT_WIDTH_SMALL));
        let timer_ul = (constants::DISPLAY_SIZE.0 -
                        SSDisplay::calculate_width(constants::ELEMENT_WIDTH_SMALL,
                                                   constants::ELEMENT_GAP_SMALL),
                        0);
        let timer_lr = (timer_ul.0 +
                        SSDisplay::calculate_width(constants::ELEMENT_WIDTH_SMALL,
                                                   constants::ELEMENT_GAP_SMALL),
                        SSDisplay::calculate_height(constants::ELEMENT_WIDTH_SMALL));
        let silent_ul = (0, constants::DISPLAY_SIZE.1 - constants::SILENT_BTN_SIZE.1);
        let silent_lr = (constants::SILENT_BTN_SIZE.0, constants::DISPLAY_SIZE.1);
        let silent_offset_pos = pos.1 + constants::TARGET_SIZE.1;
        if Self::point_is_within(pos, score_ul, score_lr) ||
           Self::point_is_within((pos.0 + constants::TARGET_SIZE.0, pos.1),
                                 timer_ul,
                                 timer_lr) ||
           (Self::point_is_within((pos.0, silent_offset_pos), silent_ul, silent_lr) &&
            silent_offset_pos < constants::DISPLAY_SIZE.1) {
            return false;
        }

        for hero in existing_hero {
            if Self::are_overlapping_targets(hero, pos) {
                return false;
            }
        }
        for evil in existing_evil {
            if Self::are_overlapping_targets(evil, pos) {
                return false;
            }
        }
        true
    }
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

