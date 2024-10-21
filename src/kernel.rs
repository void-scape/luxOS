use crate::{
    framebuffer::*,
    gdt, idt,
    interrupt::{self, InterruptLookup},
    multiboot::MultibootHeader,
    port::PortManager,
    ps2::{KeyCode, KeyState, KeyboardInput, Ps2Keyboard},
    serial_println,
    time::Rtc,
};

#[allow(unused)]
pub struct Kernel {
    interrupt_lookup: &'static InterruptLookup,
    port_manager: PortManager,
    frame_buf: FrameBuffer,
    keyboard: Ps2Keyboard,
}

fn check_interrupt_state(location: &str) {
    unsafe {
        let mut flags: u32;
        core::arch::asm!("pushf; pop {}", out(reg) flags);
        serial_println!(
            "Interrupt state at {}: IF={}",
            location,
            (flags & (1 << 9)) != 0
        );
    }
}

impl Kernel {
    pub fn new(multiboot_header: &MultibootHeader) -> Self {
        interrupt::InterruptGuard::run(|| {
            let mut port_manager = PortManager::default();
            gdt::init();
            let interrupt_lookup = idt::init();
            interrupt::init(&mut port_manager);

            Rtc::enable_irq(&mut port_manager, interrupt_lookup);
            let keyboard = Ps2Keyboard::new(&mut port_manager, interrupt_lookup);

            interrupt_lookup.register_handler(interrupt::InterruptHandler::Pic(
                interrupt::PicHandler::new(interrupt::IrqId::Pic1(0), move || {
                    // Prevent PIT interrupt warnings
                }),
            ));

            let frame_buf = FrameBuffer::new(multiboot_header);

            crate::info!("kernel initialized");

            Self {
                port_manager,
                interrupt_lookup,
                frame_buf,
                keyboard,
            }
        })
    }

    pub fn run(&mut self) {
        #[allow(clippy::empty_loop)]
        loop {}
    }

    pub fn square_demo(&mut self) {
        let mut red_rect = Rect::new(
            Point::new(0, 500),
            Dimensions::new(200, 200),
            Color::new_rgb(255, 0, 0),
        );

        let mut yellow_rect = Rect::new(
            Point::new(400, 0),
            Dimensions::new(200, 200),
            Color::new_rgb(255, 255, 0),
        );

        let mut player_rect = Rect::new(
            Point::new(400, 400),
            Dimensions::new(200, 200),
            Color::new_rgb(255, 255, 255),
        );

        let mut last_key_pressed: Option<KeyCode> = None;

        // #[allow(clippy::empty_loop)]
        loop {
            red_rect.tl.x += 1;
            if red_rect.tl.x > 400 {
                red_rect.tl.x = 0;
            }

            yellow_rect.tl.y += 4;
            if yellow_rect.tl.y > 400 {
                yellow_rect.tl.y = 0;
            }

            self.keyboard.read_input_with(|input: KeyboardInput| {
                // crate::info!("reading: {:?}", input);
                if input.state == KeyState::Pressed {
                    last_key_pressed = Some(input.key_code);
                } else if Some(input.key_code) == last_key_pressed {
                    last_key_pressed = None;
                }
            });

            const PLAYER_SPEED: isize = 16;
            if let Some(last_key_pressed) = &last_key_pressed {
                match last_key_pressed {
                    KeyCode::KeyW => player_rect.tl.y -= PLAYER_SPEED,
                    KeyCode::KeyS => player_rect.tl.y += PLAYER_SPEED,
                    KeyCode::KeyD => player_rect.tl.x += PLAYER_SPEED,
                    KeyCode::KeyA => player_rect.tl.x -= PLAYER_SPEED,
                    _ => (),
                }
            }

            self.frame_buf.present_frame(|frame: &mut FrameBuffer| {
                frame.draw_rect(&red_rect);
                frame.draw_rect(&yellow_rect);
                frame.draw_rect(&player_rect);

                frame.draw_rect(&Rect::new(
                    Point::new(frame.width as isize - 100, 0),
                    Dimensions::new(200, 200),
                    Color::new_rgb(0, 255, 0),
                ));

                frame.draw_rect(&Rect::new(
                    Point::new(-100, -100),
                    Dimensions::new(200, 200),
                    Color::new_rgb(0, 0, 255),
                ));

                frame.draw_rect(&Rect::new(
                    Point::new(frame.width as isize - 100, frame.height as isize - 100),
                    Dimensions::new(200, 200),
                    Color::new_rgb(255, 0, 255),
                ));
            });
        }
    }
}
