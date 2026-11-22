use std::sync::Mutex;
use std::thread;

const MAX_WORKERS: usize = 16;

pub fn map_bounded<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> Option<R> + Sync,
{
    if items.is_empty() {
        return Vec::new();
    }
    let workers = items.len().min(MAX_WORKERS);
    let cursor = Mutex::new(0usize);
    let results = Mutex::new(Vec::new());

    thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| loop {
                let idx = {
                    let Ok(mut c) = cursor.lock() else { break };
                    if *c >= items.len() {
                        break;
                    }
                    let i = *c;
                    *c += 1;
                    i
                };
                if let Some(res) = f(&items[idx]) {
                    if let Ok(mut out) = results.lock() {
                        out.push(res);
                    }
                }
            });
        }
    });

    results.into_inner().unwrap_or_default()
}
