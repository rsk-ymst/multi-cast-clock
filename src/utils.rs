use std::env::args;

pub mod args {
    use std::env::args;

    /* バイナリ実行時の引数取得を簡略化する関数: usize */
    pub fn get_as_usize(idx: usize) -> usize {
        args()
            .collect::<Vec<String>>()
            .get(idx)
            .unwrap()
            .as_str()
            .parse()
            .unwrap()
    }

    /* バイナリ実行時の引数取得を簡略化する関数: usize */
    pub fn get_as_String(idx: usize) -> String {
        args()
            .collect::<Vec<String>>()
            .get(idx)
            .unwrap()
            .to_string()
    }
}

pub fn get_arg_as_usize(idx: usize) -> usize {
    args()
        .collect::<Vec<String>>()
        .get(idx)
        .unwrap()
        .as_str()
        .parse()
        .unwrap()
}
