use std::{time::Duration, sync::{Mutex, Arc}, thread::{self, JoinHandle}};

pub const TICK_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Default)]
pub struct LogicClock {
    pub clock: f64,
}

impl LogicClock {
    pub fn tick(&mut self) {
        self.clock += 0.1;
    }
}

/* タイマーのカウントを開始する関数。スレッドの引数を返す */
pub fn start_clock_tick(shared_value: &Arc<Mutex<LogicClock>>) -> JoinHandle<()> {
    let clock = Arc::clone(shared_value);

    thread::spawn(move || {
        loop {
            // clock.lock().unwrap().tick();
            match clock.lock() {
                Ok(mut guard) => guard.tick(),
                Err(_) => panic!(),
            };
            thread::sleep(TICK_INTERVAL); // 普通にloopを回すとタイマーの値が大きくなりすぎるので、sleepをはさむ

            #[cfg(debug_assertions)]
            println!("clock: {:.1}", clock.lock().unwrap().clock);
        }
    })
}
