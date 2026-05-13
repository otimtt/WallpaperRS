use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct WallpaperState {
    pub paused:      AtomicBool,
    pub should_stop: AtomicBool,
    pub target_fps:  AtomicU32,
}

impl WallpaperState {
    pub fn new(target_fps: u32) -> Arc<Self> {
        Arc::new(Self {
            paused:      AtomicBool::new(false),
            should_stop: AtomicBool::new(false),
            target_fps:  AtomicU32::new(target_fps),
        })
    }

    pub fn pause(&self) { self.paused.store(true, Ordering::Relaxed); }
    pub fn resume(&self) { self.paused.store(false, Ordering::Relaxed); }
    pub fn stop(&self)  { self.should_stop.store(true, Ordering::Relaxed); }

    pub fn is_paused(&self)  -> bool { self.paused.load(Ordering::Relaxed) }
    pub fn is_stopped(&self) -> bool { self.should_stop.load(Ordering::Relaxed) }
    pub fn fps(&self) -> u32 { self.target_fps.load(Ordering::Relaxed) }
}
