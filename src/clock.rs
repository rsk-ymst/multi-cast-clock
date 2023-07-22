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
        }
    })
}

/*
    1 ~ 3 tick分のスリープをランダムで生成する。
    常に等間隔でスリープさせると、レシーバ間のタイムスタンプにばらつきが生じないため、
    IDによって間隔に差分を生じせる。
*/
pub fn sleep_random_interval(receiver_id: usize) {
    let random_num: u64 = if receiver_id == 2 {
        rand::thread_rng().gen_range(3..=6)
    } else {
        rand::thread_rng().gen_range(1..=2)
    };
    thread::sleep(Duration::from_millis(100 * random_num))
}

pub fn adjust_time_clock(current_time: &Arc<Mutex<LogicClock>>, received_time: f64) {
    /* もし未来からのメッセージが来ていたら、時刻を調整 */
    match current_time.lock() {
        Ok(mut current) => {
            if received_time > current.clock {
                current.clock = received_time + 1.0;
            }
        }
        Err(_) => panic!(),
    };

    // drop(current); // Mutexロック解除
}

/* 論理クロックの現時刻を取得する関数 */
pub fn get_current_timestamp(value: &Arc<Mutex<LogicClock>>) -> Option<f64> {
    let current = value.lock().unwrap();
    let res = current.clock;

    drop(current); // Mutexロック解除
    Some(res)
}
