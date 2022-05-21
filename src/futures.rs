use core::future::Future;

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    crate::process::spawn_kernel(future);
}


/// Schedules future on main kernel loop
macro_rules! spawn {
    ($arg:expr) => {
        crate::futures::spawn($arg)
    };
}
