use ps2::Controller;
use ps2::flags::{ControllerConfigFlags, KeyboardLedFlags, MouseMovementFlags};
use x86_64::structures::idt::InterruptStackFrame;
use spin::{Lazy, RwLock, Mutex};
use crate::driver::keyboard::KeyboardMessage;
use crate::flow::{Producer, Sender, FlowManager};
use alloc::sync::Arc;
use pc_keyboard::{Keyboard, layouts, ScancodeSet2, HandleControl, DecodedKey, KeyCode, KeyState};
use crate::{futures, interrupts};
use crate::interrupts::Irq;
use ps2::error::ControllerError;
use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::Deref;
use crate::driver::mouse::MouseMessage;
use fixedbitset::FixedBitSet;

static KEYBOARD_SENDER: Lazy<Arc<RwLock<Producer<KeyboardMessage>>>> = Lazy::new(|| Arc::new(RwLock::new(Producer::new())));
static MOUSE_SENDER: Lazy<Arc<RwLock<Producer<MouseMessage>>>> = Lazy::new(|| Arc::new(RwLock::new(Producer::new())));
static INT_CONTROLLER: Lazy<Mutex<Controller>> = Lazy::new(|| Mutex::new(unsafe { Controller::new() }));
static KEYBOARD_PARSER: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet2>>> = Lazy::new(|| Mutex::new(
    Keyboard::new(
        layouts::Us104Key,
        ScancodeSet2,
        HandleControl::Ignore
    )
));
static CAPS_STATE: AtomicBool = AtomicBool::new(false);
static NUM_STATE: AtomicBool = AtomicBool::new(false);
static SCROLL_STATE: AtomicBool = AtomicBool::new(false);

pub fn init() -> Result<(), ControllerError> {
    kblog!("PS/2", "Starting PS/2 devices");
    let mut controller = unsafe { Controller::new() };

    // Step 3: Disable devices
    controller.disable_keyboard()?;
    controller.disable_mouse()?;

    // Step 4: Flush data buffer
    let _ = controller.read_data();

    // Step 5: Set config
    let mut config = controller.read_config()?;
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
            | ControllerConfigFlags::ENABLE_TRANSLATE,
        false,
    );
    controller.write_config(config)?;

    // Step 6: Controller self-test
    controller.test_controller()?;
    // Write config again in case of controller reset
    controller.write_config(config)?;

    // Step 7: Determine if there are 2 devices
    let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
        controller.enable_mouse()?;
        config = controller.read_config()?;
        // If mouse is working, this should now be unset
        !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
    } else {
        false
    };
    // Disable mouse. If there's no mouse, this is ignored
    controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = controller.test_keyboard().is_ok();
    let mouse_works = has_mouse && controller.test_mouse().is_ok();

    // Step 9 - 10: Enable and reset devices
    config = controller.read_config()?;
    if keyboard_works {
        controller.enable_keyboard()?;
        config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
        config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
        controller.keyboard().reset_and_self_test().unwrap();

        let sender = KEYBOARD_SENDER.deref();
        FlowManager::register_endpoint("/dev/ps2/keyboard", sender.clone(), None);
        let int = Irq::from_raw(1).map_to_int(0);
        interrupts::set_handler(int, keyboard_handler);
        kblog!("PS/2", "PS/2 keyboard started");
    }
    if mouse_works {
        controller.enable_mouse()?;
        config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
        config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
        controller.mouse().reset_and_self_test().unwrap();
        // This will start streaming events from the mouse
        controller.mouse().enable_data_reporting().unwrap();

        let sender = MOUSE_SENDER.deref();
        FlowManager::register_endpoint("/dev/ps2/mouse", sender.clone(), None);
        let int = Irq::from_raw(12).map_to_int(0);
        interrupts::set_handler(int, mouse_handler);
        kblog!("PS/2", "PS/2 mouse started");
    }

    // Write last configuration to enable devices and interrupts
    controller.write_config(config)?;
    Ok(())
}

async fn send_decoded(decoded: DecodedKey) {
    KEYBOARD_SENDER.write().send(KeyboardMessage {
        key: decoded,
    }).await;
}

async fn send_mouse(packet: (MouseMovementFlags, i16, i16)) {
    let mut bitset = FixedBitSet::with_capacity(3);
    if packet.0.contains(MouseMovementFlags::LEFT_BUTTON_PRESSED) {
        bitset.set(0, true);
    }
    if packet.0.contains(MouseMovementFlags::MIDDLE_BUTTON_PRESSED) {
        bitset.set(1, true);
    }
    if packet.0.contains(MouseMovementFlags::RIGHT_BUTTON_PRESSED) {
        bitset.set(2, true);
    }
    MOUSE_SENDER.write().send(MouseMessage {
        keys_pressed: bitset,
        movement_x: packet.1,
        movement_y: packet.2
    }).await;
}

int_handler!(noint keyboard_handler |_: InterruptStackFrame| {
    let mut controller = INT_CONTROLLER.lock();
    // ignore timeouts
    if let Ok(byte) = controller.read_data() {
        let mut keyboard = KEYBOARD_PARSER.lock();
        if let Ok(key) = keyboard.add_byte(byte) {
            if let Some(key) = key {
                if let Some(decoded) = keyboard.process_keyevent(key.clone()) {
                    futures::spawn(send_decoded(decoded))
                }
                let change_led = match key.code {
                    KeyCode::CapsLock => {
                        CAPS_STATE.store(match key.state {
                            KeyState::Up => false,
                            KeyState::Down => true
                        }, Ordering::SeqCst);
                        true
                    }
                    KeyCode::NumpadLock => {
                        NUM_STATE.store(match key.state {
                            KeyState::Up => false,
                            KeyState::Down => true
                        }, Ordering::SeqCst);
                        true
                    }
                    KeyCode::ScrollLock => {
                        SCROLL_STATE.store(match key.state {
                            KeyState::Up => false,
                            KeyState::Down => true
                        }, Ordering::SeqCst);
                        true
                    }
                    _ => false
                };
                if change_led {
                    let mut flags = KeyboardLedFlags::empty();
                    if SCROLL_STATE.load(Ordering::SeqCst) {
                        flags |= KeyboardLedFlags::SCROLL_LOCK;
                    }
                    if CAPS_STATE.load(Ordering::SeqCst) {
                        flags |= KeyboardLedFlags::CAPS_LOCK;
                    }
                    if NUM_STATE.load(Ordering::SeqCst) {
                        flags |= KeyboardLedFlags::NUM_LOCK;
                    }
                    if let Err(_) = controller.keyboard().set_leds(flags) {
                        // ignore timeout
                    }
                }
            }
        }
    }
});

int_handler!(noint mouse_handler |_: InterruptStackFrame| {
    let mut controller = INT_CONTROLLER.lock();
    // ignore timeouts
    if let Ok(packet) = controller.mouse().read_data_packet() {
        futures::spawn(send_mouse(packet));
    }
});