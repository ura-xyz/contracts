pub fn get_current_epoch(current_time: u64, epoch_start_time: u64, epoch_length: u64) -> u64 {
    let seconds_since_start = current_time - epoch_start_time;
    return seconds_since_start / epoch_length;
}
