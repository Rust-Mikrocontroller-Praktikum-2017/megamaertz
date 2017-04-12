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
pub mod constants;
pub mod game;

use stm32f7::{system_clock, sdram, lcd, i2c, audio, touch, board, embedded};
use collections::vec::Vec;
use seven_segment::SSDisplay;
use embedded::interfaces::gpio::Gpio; // {self, Gpio} for use with button

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

#[inline(never)]
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

    // button controller for reset button
    // let button_pin = (gpio::Port::PortI, gpio::Pin::Pin11);
    // let button = gpio.to_input(button_pin, gpio::Resistor::NoPull)
    //     .expect("button pin already in use");

    // init sdram (needed for display buffer)
    sdram::init(rcc, fmc, &mut gpio);

    // lcd controller
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    lcd.clear_screen();
    lcd.set_background_color(lcd::Color::rgb(255, 193, 37));

    // i2c
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
    let mut seed = random_gen.poll_and_get();
    while seed.is_err() {
        seed = random_gen.poll_and_get();
    }
    let rand = random::MTRng32::new(seed.unwrap());

    //renderer
    let mut rend = renderer::Renderer::new(&mut lcd);
    rend.draw_dump_bg(0,
                      0,
                      (constants::DISPLAY_SIZE.0, constants::DISPLAY_SIZE.1),
                      constants::BACKGROUND);

    let tick = system_clock::ticks();

    //create and init game
    let mut game = game::Game {
        evil_targets: Vec::new(),
        hero_targets: Vec::new(),
        rend: &mut rend,
        score: 0,
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
    };

    // draw game banner
    game.draw_game_banner();
    loop {
        if !touch::touches(&mut i2c_3).unwrap().is_empty() {
            break;
        }
    }

    // switch to start screen
    game.draw_start_banner();
    let mut game_running = false;
    let mut touches_to_start = 1;

    // loop game
    loop {
        let mut touches: Vec<(u16, u16)> = Vec::new();
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            touches.push((touch.x, touch.y));
        }

        if game_running {
            if 0 < game.update_countdown() {
                game.draw_missing_targets();
                game.process_shooting(sai_2, touches);
                game.purge_old_targets();
            } else {
                game.game_over();
                game_running = false;
            }
        } else if !touches.is_empty() && touches_to_start > 2 {
            touches_to_start -= 1;
            stm32f7::system_clock::wait(250);
        } else if !touches.is_empty() && touches_to_start == 2 {
            game.draw_start_banner();
            touches_to_start -= 1;
        } else if !touches.is_empty() && touches_to_start == 1 {
            game.start(touches.pop().unwrap());
            game_running = true;
            touches_to_start = 3;
        }
    }

}
