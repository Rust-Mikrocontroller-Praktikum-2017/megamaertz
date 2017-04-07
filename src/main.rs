#![feature(alloc, collections)]

#![no_std]
#![no_main]

// initialization routing for .data and .bss
extern crate r0;
extern crate stm32f7_discovery as stm32f7;

extern crate collections;
extern crate alloc;

pub mod renderer;
pub mod seven_segment;
pub mod random;
pub mod shooter;

use stm32f7::{system_clock, sdram, lcd, i2c, audio, touch, board, embedded};
use stm32f7::board::sai::Sai;
use collections::vec::Vec;

// static TRUMP: &'static [u8] = include_bytes!("../pics/trump.dump");
// static TRUMP_SIZE: (u16, u16) = (42, 50);
static TRUMP: &'static [u8] = include_bytes!("../pics/trump_cartoon.dump");
static TRUMP_SIZE: (u16, u16) = (50, 50);

const DISPLAY_SIZE: (u16, u16) = (480, 272);
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

    // enable floating point unit
    let scb = stm32f7::cortex_m::peripheral::scb_mut();
    scb.cpacr.modify(|v| v | 0b1111 << 20);

    main(board::hw());
}

fn main(hw: board::Hardware) -> ! {
    let board::Hardware { rcc,
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
    lcd.set_background_color(lcd::Color::rgb(0, 200, 0));

    //i2c
    i2c::init_pins_and_clocks(rcc, &mut gpio);
    let mut i2c_3 = i2c::init(i2c_3);
    touch::check_family_id(&mut i2c_3).unwrap();

    // sai and stereo microphone
    audio::init_sai_2_pins(&mut gpio);
    audio::init_sai_2(sai_2, rcc);
    assert!(audio::init_wm8994(&mut i2c_3).is_ok());

    // initialize random number coordinator
    let mut rand = random::CmwcState::new();

    //renderer
    let mut rend = renderer::Renderer::new(&mut lcd);
    rend.draw_dump_bg(0, 0, DISPLAY_SIZE, &BACKGROUND);

    let mut ss_display = seven_segment::SSDisplay::new(0, 0);

    // array of all targets
    let mut targets: Vec<shooter::Target> = Vec::new();
    let mut active_target_count = 0;

    let mut last_ssd_render_time = system_clock::ticks();
    let mut counter: u16 = 0;


    loop {
        let tick = system_clock::ticks();
        if tick - last_ssd_render_time >= 1000 {
            counter = (counter % core::u16::MAX) + 1;
            let ss_pixel = ss_display.render(counter, 0x8000);
            rend.draw_u16_tuple(ss_pixel.as_slice());
            last_ssd_render_time = tick;
        }

        if vol_limit_reached(sai_2) {
            // draw debug square
            rend.draw_u16(100,100,25, &[0x8000 as u16; 625]);
        } else {
            // undraw debug square
            rend.draw_u16(100,100,25, &[0x0000 as u16; 625]);
        }

        // rendering random positioned trumps
        if active_target_count < 5 {
            let pos : (u16, u16) = rand.get_random_pos(DISPLAY_SIZE.0, DISPLAY_SIZE.1);
            let new_target = shooter::Target::new(pos.0, pos.1, TRUMP_SIZE.0, TRUMP_SIZE.1);
            rend.draw_dump(pos.0, pos.1, TRUMP_SIZE, &TRUMP);
            targets.push(new_target);
            active_target_count += 1;
        }

        // check for hit and render cursor
        rend.remove_last_cursor();
        let mut touches : Vec<(u16, u16)> = Vec::new();
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            rend.cursor(touch.x, touch.y);
            touches.push((touch.x, touch.y));
        }
        let mut hitted_targets = shooter::Target::check_for_hit(&mut targets, &touches);
        &hitted_targets.sort();
        for hit_index in hitted_targets.iter().rev() {
            targets.remove(*hit_index);
            active_target_count -= 1;
        }
    }
}

fn vol_limit_reached(sai_2: &'static Sai) -> bool {
    while !sai_2.bsr.read().freq() {} // fifo_request_flag
    let data0 = sai_2.bdr.read().data() as i16 as i32;
    while !sai_2.bsr.read().freq() {} // fifo_request_flag
    let data1 = sai_2.bdr.read().data() as i16 as i32;

    let mic_data =
        if data0.abs() > data1.abs() {
            data0.abs() as u16
        } else {
            data1.abs() as u16
        };

    // mic_data reprents our "volume". Magic number 420 after testing.
    let blaze_it = 420;
    if mic_data > blaze_it {
        true
    } else {
        false
    }
}
