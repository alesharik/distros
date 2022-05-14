use crate::driver::TtyMessage;
use crate::flow::{FlowManager, FlowManagerError};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use async_trait::async_trait;
use core::any::TypeId;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::lock::BiLock;
use libkernel::flow::{AnyConsumer, Consumer, Message, Subscription};

struct Sub {
    line: BiLock<String>,
}

struct CatSub {}

struct Load {}

impl Future for Load {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        loop {

        }
    }
}

impl Unpin for Load {}


impl Sub {
    fn new() -> Sub {
        Sub {
            line: BiLock::new("".to_owned()).0,
        }
    }

    fn print(&self, s: &str) {
        FlowManager::send_async("/dev/tty/vga", TtyMessage::new(s));
    }

    fn new_line(&self) {
        self.print("\n# ");
    }

    fn init(&self) {
        self.print("# ");
    }

    async fn load() {
        loop {}
    }

    fn read_command(&self, line: &str) {
        let parts = line.split(' ').collect::<Vec<_>>();
        let (command, arguments) = parts.split_first().unwrap();
        match *command {
            "test" => {
                for x in 31..38 {
                    self.print(&format!("\x1b[{}m {} ", x, x));
                }
                self.print("\n\x1B[33m> YAY!!");
            }
            "ls" => {
                for x in FlowManager::list(arguments[0]) {
                    self.print(&format!("{}\n", x))
                }
            }
            "cat" => match FlowManager::subscribe(arguments[0], Box::new(CatSub {})) {
                Ok(sub) => {
                    self.print(&format!("Sub id = {}", sub.get_id()));
                    core::mem::forget(sub);
                }
                Err(e) => {
                    self.print(&format!("\x1b[31mError is {:?}\x1B[37m>", e));
                }
            },
            "load" => {
                spawn!(Load {})
            },
            "" => {}
            _ => {
                self.print(&format!("Unknown command {}", line));
            }
        }
    }
}

#[async_trait]
unsafe impl AnyConsumer for CatSub {
    fn check_type(&self, _msg_type: &TypeId) -> bool {
        true
    }

    async fn consume_msg(&self, message: &dyn Message) {
        FlowManager::send_async("/dev/tty/vga", TtyMessage::new(&format!("{:?}", message)));
    }

    async fn close_consumer(&self, _sub: &dyn Subscription) {
        FlowManager::send_async("/dev/tty/vga", TtyMessage::new("CLOSED"));
    }
}

#[async_trait]
impl Consumer for Sub {
    type Msg = TtyMessage;

    async fn consume(&self, message: &TtyMessage) {
        let mut line = self.line.lock().await;
        let input = message.to_string();
        if input == "\n" {
            self.print("\n");
            self.read_command(&line);
            *line = "".to_owned();
            self.new_line();
        } else {
            *line += &input;
            self.print(&input);
        }
    }

    async fn close(&self, _sub: &dyn Subscription) {}
}

pub fn init() -> Result<(), FlowManagerError> {
    let sub = Sub::new();
    sub.init();
    let sub = FlowManager::subscribe("/dev/tty/vga", Box::new(sub))?;
    core::mem::forget(sub); // do not unsub never
    Ok(())
}
