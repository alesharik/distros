use crate::driver::keyboard::KeyboardMessage;
use crate::driver::tty::{TtyMessage, TtyWriter};
use crate::flow::{Consumer, Producer, Provider, Sender, Subscription, AnyConsumer};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use pc_keyboard::{DecodedKey, KeyCode};
use spin::Mutex;
use vte::ansi::{Handler, Processor};

pub struct Stdin(Producer<TtyMessage>);

impl Stdin {
    pub fn new() -> Self {
        Stdin(Producer::new())
    }

    pub async fn send(&mut self, s: &str) {
        let message = TtyMessage(s.to_owned());
        self.0.send(message).await;
    }
}

impl Provider for Stdin {
    fn add_consumer(&mut self, consumer: Box<dyn AnyConsumer>) -> Box<dyn Subscription> {
        self.0.add_consumer(consumer)
    }
}

impl TtyWriter for Stdin {
    fn write_back(&mut self, s: &str) {
        let message = TtyMessage(s.to_owned());
        self.0.send_async(message);
    }
}

pub struct StdinKeyboardConsumer {
    stdin: Arc<Mutex<Stdin>>,
}

impl StdinKeyboardConsumer {
    pub fn new(stdin: Arc<Mutex<Stdin>>) -> Self {
        StdinKeyboardConsumer { stdin }
    }
}

#[async_trait]
impl Consumer for StdinKeyboardConsumer {
    type Msg = KeyboardMessage;

    async fn consume(&self, message: &KeyboardMessage) {
        info!("MSG {:?}", message);
        match message.key {
            DecodedKey::Unicode(code) => {
                let message = format!("{}", code);
                self.stdin.lock().send(&message).await;
            }
            DecodedKey::RawKey(code) => match code {
                KeyCode::ArrowLeft => self.stdin.lock().send("\x1B[1D").await,
                KeyCode::ArrowRight => self.stdin.lock().send("\x1B[1C").await,
                KeyCode::Backspace => self.stdin.lock().send("^A").await,
                _ => {
                    return;
                }
            },
        }
    }

    async fn close(&self, _sub: &Box<dyn Subscription>) {}
}

pub struct Stdout<H: Handler<Stdin>> {
    handler: H,
    processor: Processor,
    stdin: Arc<Mutex<Stdin>>,
}

impl<H: Handler<Stdin>> Stdout<H> {
    pub fn new(handler: H, stdin: Arc<Mutex<Stdin>>) -> Self {
        Stdout {
            handler,
            stdin,
            processor: Processor::new(),
        }
    }
}

#[async_trait]
impl<H: Handler<Stdin> + Send + Sync> Sender for Stdout<H> {
    type Msg = TtyMessage;

    async fn send(&mut self, message: TtyMessage) {
        let mut guard = self.stdin.lock();
        for c in message.0.chars() {
            self.processor
                .advance(&mut self.handler, c as u8, &mut guard);
        }
    }
}
