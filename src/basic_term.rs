use crate::driver::TtyMessage;
use crate::flow::{Consumer, FlowManager, FlowManagerError, Subscription};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use async_trait::async_trait;
use futures::lock::BiLock;

struct Sub {
    line: BiLock<String>,
}

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
}

#[async_trait]
impl Consumer<TtyMessage> for Sub {
    async fn consume(&self, message: &TtyMessage) {
        let mut line = self.line.lock().await;
        let input = message.to_string();
        if input == "\n" {
            if line.starts_with("test") {
                for x in 31..38 {
                    self.print(&format!("\x1b[{}m {} ", x, x));
                }
                self.print("\n\x1B[33m> YAY!!");
            }
            *line = "".to_owned();
            self.new_line();
        } else {
            *line += &input;
            self.print(&input);
        }
    }

    async fn close(&self, _sub: &Box<dyn Subscription>) {}
}

pub fn init() -> Result<(), FlowManagerError> {
    let sub = Sub::new();
    sub.init();
    let sub = FlowManager::subscribe("/dev/tty/vga", Box::new(sub))?;
    core::mem::forget(sub); // do not unsub never
    Ok(())
}