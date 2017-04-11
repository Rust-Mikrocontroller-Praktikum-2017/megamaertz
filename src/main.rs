#![feature(alloc, collections)]

#![no_std]
#![no_main]

// initialization routing for .data and .bss
extern crate r0;
extern crate stm32f7_discovery as stm32f7;

extern crate collections;
extern crate alloc;
extern crate bit_field;

pub mod renderer;
pub mod seven_segment;
pub mod random;
pub mod shooter;
pub mod constants;
pub mod game;

use stm32f7::{system_clock, sdram, lcd, i2c, audio, touch, board, embedded};
use collections::vec::Vec;
use shooter::Target;
use random::Rng;

static TRUMP: &'static [u8] = include_bytes!("../pics/trump_cartoon.dump");
static SUPER_TRUMP: &'static [u8] = include_bytes!("../pics/mexican_trump_head.dump");
static MEXICAN: &'static [u8] = include_bytes!("../pics/mexican_cartoon.dump");
static BACKGROUND: &'static [u8] = include_bytes!("../pics/background.dump");

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        static __DATA_LOAD: u32;
        static __DATA_END: u32;
        static mut __DATA_START: u32;
        static mut __BSS_START: u32;
        static mut __BSS_END: u32;
    }

    let data_load = &__DATA_LOAD;
    let data_start = &mut __DATA_START;
    let data_end = &__DATA_END;
    let bss_start = &mut __BSS_START;
    let bss_end = &__BSS_END;

    // initializes the .data section
    //(copy the data segment initializers from flash to RAM)
    r0::init_data(data_start, data_end, data_load);
    // zeroes the .bss section
    r0::zero_bss(bss_start, bss_end);

    stm32f7::heap::init();

    main(board::hw());
}

fn main(hw: board::Hardware) -> ! {
    let board::Hardware { rcc,
                          rng,
                          pwr,
                          flash,
                          fmc,
                          ltdc,
                          gpio_a,
                          gpio_b,
                          gpio_c,
                          gpio_d,
                          gpio_e,
                          gpio_f,
                          gpio_g,
                          gpio_h,
                          gpio_i,
                          gpio_j,
                          gpio_k,
                          i2c_3,
                          sai_2,
                          .. } = hw;

    use embedded::interfaces::gpio::{self, Gpio};
    let mut gpio = Gpio::new(gpio_a,
                             gpio_b,
                             gpio_c,
                             gpio_d,
                             gpio_e,
                             gpio_f,
                             gpio_g,
                             gpio_h,
                             gpio_i,
                             gpio_j,
                             gpio_k);

    system_clock::init(rcc, pwr, flash);
    // enable all gpio ports
    rcc.ahb1enr.update(|r| {
        r.set_gpioaen(true);
        r.set_gpioben(true);
        r.set_gpiocen(true);
        r.set_gpioden(true);
        r.set_gpioeen(true);
        r.set_gpiofen(true);
        r.set_gpiogen(true);
        r.set_gpiohen(true);
        r.set_gpioien(true);
        r.set_gpiojen(true);
        r.set_gpioken(true);
    });


    // configure led pin as output pin
    let led_pin = (gpio::Port::PortI, gpio::Pin::Pin1);
    let mut led = gpio.to_output(led_pin,
                   gpio::OutputType::PushPull,
                   gpio::OutputSpeed::Low,
                   gpio::Resistor::NoPull)
        .expect("led pin already in use");

    // turn led on
    led.set(true);

    // init sdram (needed for display buffer)
    sdram::init(rcc, fmc, &mut gpio);

    // lcd controller
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    lcd.clear_screen();
    lcd.set_background_color(lcd::Color::rgb(255, 193, 37));

    //i2c
    i2c::init_pins_and_clocks(rcc, &mut gpio);
    let mut i2c_3 = i2c::init(i2c_3);
    touch::check_family_id(&mut i2c_3).unwrap();

    // sai and stereo microphone
    audio::init_sai_2_pins(&mut gpio);
    audio::init_sai_2(sai_2, rcc);
    assert!(audio::init_wm8994(&mut i2c_3).is_ok());

    // initialize random number generator and pseudo
    // random number generator
    let mut random_gen = stm32f7::random::Rng::init(rng, rcc).unwrap();
    let seed = random_gen.poll_and_get().unwrap();
    let mut rand = random::MTRng32::new(seed);

    //renderer
    let mut rend = renderer::Renderer::new(&mut lcd);
    rend.draw_dump_bg(0,
                      0,
                      (constants::DISPLAY_SIZE.0, constants::DISPLAY_SIZE.1),
                      BACKGROUND);

    // coundown
    let mut ss_display = seven_segment::SSDisplay::new(480 - seven_segment::SSDisplay::get_width(),
                                                       0);

    // score
    let mut score: u16 = 0;
    let mut ss_hs_display = seven_segment::SSDisplay::new(0, 0);
    let red: u16 = renderer::RGBColor::from_rgb(255, 0, 0);
    let green: u16 = renderer::RGBColor::from_rgb(0, 255, 0);
    ss_hs_display.render(score, 0x8000, &mut rend);

    // array of all evil_targets
    let mut evil_targets: Vec<shooter::Target> = Vec::new();
    let mut hero_targets: Vec<shooter::Target> = Vec::new();

    let mut last_ssd_render_time = system_clock::ticks();
    let mut last_super_trump_render_time = system_clock::ticks();
    let mut counter: u16 = 0;

    loop {
        // seven segments display for countdown
        let tick = system_clock::ticks();
        if tick - last_ssd_render_time >= 1000 {
            counter = (counter % core::u16::MAX) + 1;
            ss_display.render(counter, 0x8000, &mut rend);
            last_ssd_render_time = tick;
        }

        // rendering random positioned evil evil_targets (trumps)
        while evil_targets.len() < 5 {
            let lifetime = game::get_rnd_lifetime(&mut rand);
            let pos: (u16, u16) = game::get_rnd_pos(&mut rand, &hero_targets, &evil_targets);
            let evil_target = Target::new(pos.0,
                                          pos.1,
                                          constants::TARGET_SIZE_50.0,
                                          constants::TARGET_SIZE_50.1,
                                          50,
                                          tick,
                                          lifetime);
            let super_evil_target = shooter::Target::new(pos.0,
                                                   pos.1,
                                                   constants::TARGET_SIZE_50.0,
                                                   constants::TARGET_SIZE_50.1,
                                                   100,
                                                   tick,
                                                   2000);
            if tick - last_super_trump_render_time >= 8000 + (rand.rand() as usize % 3000) {
                rend.draw_dump(pos.0, pos.1, constants::TARGET_SIZE_50, SUPER_TRUMP);
                last_super_trump_render_time = tick;
                evil_targets.push(super_evil_target);
            } else {
                rend.draw_dump(pos.0, pos.1, constants::TARGET_SIZE_50, TRUMP);
                evil_targets.push(evil_target);
            }
        }

        // rendering random positioned hero evil_targets (mexicans)
        while hero_targets.len() < 3 {
            let lifetime = game::get_rnd_lifetime(&mut rand);
            let pos: (u16, u16) = game::get_rnd_pos(&mut rand, &hero_targets, &evil_targets);
            let hero_target = shooter::Target::new(pos.0,
                                                   pos.1,
                                                   constants::TARGET_SIZE_50.0,
                                                   constants::TARGET_SIZE_50.1,
                                                   30,
                                                   tick,
                                                   lifetime);
            rend.draw_dump(pos.0, pos.1, constants::TARGET_SIZE_50, MEXICAN);
            hero_targets.push(hero_target);
        }

        let mut touches: Vec<(u16, u16)> = Vec::new();
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            touches.push((touch.x, touch.y));
        }

        if game::vol_limit_reached(sai_2) {
            let mut hit_evil_targets = Target::check_for_hit(&mut evil_targets, &touches);
            hit_evil_targets.sort();
            for hit_index in hit_evil_targets.iter().rev() {
                let t = evil_targets.remove(*hit_index);
                rend.clear(t.x, t.y, (t.width, t.height));
                score += t.bounty;
                ss_hs_display.render(score, green, &mut rend);
            }
            let mut hit_hero_targets = Target::check_for_hit(&mut hero_targets, &touches);
            hit_hero_targets.sort();
            for hit_index in hit_hero_targets.iter().rev() {
                let t = hero_targets.remove(*hit_index);
                rend.clear(t.x, t.y, (t.width, t.height));
                score -= if score < 30 { score } else { t.bounty };
                ss_hs_display.render(score, red, &mut rend);
            }
        }

        // dont let targets live longer than lifetime secs
        for i in (0..evil_targets.len()).rev() {
            if tick - evil_targets[i].birthday > evil_targets[i].lifetime {
                let t = evil_targets.remove(i);
                rend.clear(t.x, t.y, (t.width, t.height));
            }
        }

        for i in (0..hero_targets.len()).rev() {
            if tick - hero_targets[i].birthday > hero_targets[i].lifetime {
                let t = hero_targets.remove(i);
                rend.clear(t.x, t.y, (t.width, t.height));
            }
        }
    }
}

