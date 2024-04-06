use crate::{NiceLevel, TaskFlags, TaskId};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::pin::Pin;
use hashbrown::HashMap;

struct Task {
    id: TaskId,
    name: Option<String>,
    nice: NiceLevel,
}

pub struct TaskBuilder {
    name: Option<String>,
    nice: NiceLevel,
    flags: TaskFlags,
    executable: Pin<Box<dyn Future<Output = ()>>>,
}

impl TaskBuilder {
    pub fn kernel(task: impl Future<Output = ()> + 'static) -> Self {
        TaskBuilder {
            nice: NiceLevel::default(),
            name: None,
            flags: TaskFlags::empty(),
            executable: Box::pin(task),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn nice(mut self, nice: NiceLevel) -> Self {
        self.nice = nice;
        self
    }

    pub fn no_preempt(mut self) -> Self {
        self.flags.set(TaskFlags::NOPREEMPT, true);
        self
    }

    pub fn wrap_executable(
        mut self,
        wrapper: impl FnOnce(Pin<Box<dyn Future<Output = ()>>>) -> Pin<Box<dyn Future<Output = ()>>>,
    ) -> Self {
        self.executable = wrapper(self.executable);
        self
    }
}

pub struct TaskRegistry {
    tasks: HashMap<TaskId, Task>,
}

impl TaskRegistry {
    pub(crate) fn new() -> TaskRegistry {
        TaskRegistry {
            tasks: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, task: TaskBuilder) -> TaskId {
        let id = crate::scheduler::add(task.executable, task.nice, task.flags);
        self.tasks.insert(
            id,
            Task {
                id,
                name: task.name,
                nice: task.nice,
            },
        );
        id
    }

    pub(crate) fn remove(&mut self, task: TaskId) {
        self.tasks.remove(&task);
    }

    pub fn get_name(&self, task_id: TaskId) -> Option<Option<String>> {
        self.tasks.get(&task_id).map(|s| s.name.to_owned())
    }

    pub fn get_nice(&self, task_id: TaskId) -> Option<NiceLevel> {
        self.tasks.get(&task_id).map(|s| s.nice)
    }

    pub fn set_name(&mut self, task_id: TaskId, name: impl Into<String>) {
        self.tasks.entry(task_id).and_modify(move |x| {
            x.name = Some(name.into());
        });
    }
}
