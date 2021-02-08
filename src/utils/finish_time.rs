pub fn seconds_to_finish_time(mut seconds: i64) -> String {
    let s = seconds % 60;
    seconds /= 60;
    let m = seconds % 60;
    seconds /= 60;
    let h = seconds;

    return format!("{:0>2}:{:0>2}:{:0>2}", h, m, s);
}
