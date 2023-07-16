use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use rand::Rng;

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
            match clock.lock() {
                Ok(mut guard) => guard.tick(),
                Err(_) => panic!(),
            };
            thread::sleep(TICK_INTERVAL); // 普通にloopを回すとタイマーの値が大きくなりすぎるので、sleepをはさむ

            // #[cfg(debug_assertions)]
            // println!("clock: {:.1}", clock.lock().unwrap().clock);
        }
    })
}

/* 1 ~ 3 tick分のスリープをランダムで生成する。常に等間隔でスリープさせると、レシーバ間のタイムスタンプにばらつきが生じないため。  */
pub fn sleep_random_interval() {
    let random_num: u64 = rand::thread_rng().gen_range(1..=3);
    thread::sleep(Duration::from_millis(random_num * 100))
}
