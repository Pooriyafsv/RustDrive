use std::io::{self, Write};
pub fn print_progress(current: u64, total: u64) {
    let percentage = (current as f64 / total as f64) * 100.0;
    let filled = (percentage / 5.0) as usize;
    let bar = format!("{}{}","█".repeat(filled), "░".repeat(20 - filled));
    print!("\r[{}] {:.2}%", bar, percentage);
    io::stdout().flush().unwrap();
    if current == total {
        println!()
    }
}