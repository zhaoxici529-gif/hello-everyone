// 发行版下隐藏 Windows 控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    media_ai_workbench_lib::run()
}
