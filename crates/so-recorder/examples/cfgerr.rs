//! 临时诊断：打印配置解析错误详情。

#![allow(clippy::print_stdout)] // 诊断工具的输出就是 stdout

fn main() {
    let path = std::env::args().nth(1).expect("usage: cfgerr <toml>");
    let text = std::fs::read_to_string(&path).unwrap();
    match so_recorder::RecordingConfig::parse(&text) {
        Ok(_) => println!("OK"),
        Err(e) => println!("ERR code={} msg={e}", e.code()),
    }
}
