use super::*;

impl Stringify for Duration {
    fn stringify(&self) -> String {
        if !is_in_console_mode() {
            return self.as_millis().to_string();
        }
        if self < &Duration::from_secs(1) {
            return self.as_millis().to_string() + " ms";
        }
        let precision = 3;
        let total_secs = self.as_secs_f64();
        for (threshold, unit) in [(86400.0, "days"), (3600.0, "hr"), (60.0, "min")] {
            if total_secs >= threshold {
                let time_unit = total_secs as u128 / threshold as u128;
                let secs = total_secs % threshold;
                let mut string = format!("{} {}", time_unit, unit);
                if time_unit > 1 {
                    string += "s";
                }
                if secs >= 10.0_f64.powi(-(precision as i32)) {
                    string += " ";
                    string += &Duration::from_secs_f64(secs).stringify();
                }
                return string;
            }
        }
        let mut string = format!("{:.1$} sec", total_secs, precision);
        if total_secs > 1.0 {
            string += "s";
        }
        string
    }
}

pub fn measure_time<T>(func: impl Fn() -> T) -> T {
    let clock = Instant::now();
    let res = func();
    if is_in_console_mode() {
        println!();
    }
    println_info("Run Time", clock.elapsed().stringify());
    res
}
