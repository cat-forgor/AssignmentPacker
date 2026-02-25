use owo_colors::OwoColorize;

pub fn step(msg: &str) {
    eprintln!("  {} {msg}", "->".cyan());
}

pub fn success(msg: &str) {
    eprintln!("  {} {msg}", "ok".green().bold());
}

pub fn done(msg: &str) {
    eprintln!("{} {msg}", "done".green().bold());
}

pub fn warn(msg: &str) {
    eprintln!("{} {msg}", "warning:".yellow());
}

pub fn header(msg: &str) {
    eprintln!("{}", msg.bold());
}

pub fn kv(key: &str, val: &str) {
    eprintln!("  {}: {val}", key.dimmed());
}
