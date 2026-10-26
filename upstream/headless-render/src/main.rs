#![allow(clippy::all)]

#[path = "../../phira-render/src-tauri/src/ipc.rs"]
mod ipc;
#[path = "../../phira-render/src-tauri/src/render.rs"]
mod render;
#[path = "../../phira-render/src-tauri/src/render_plan.rs"]
mod render_plan;

use macroquad::prelude::Conf;

fn headless_conf() -> Conf {
    Conf {
        window_title: "BeatGlint Phira headless renderer".to_owned(),
        window_width: 960,
        window_height: 540,
        headless: true,
        ..Default::default()
    }
}

#[macroquad::main(headless_conf)]
async fn main() {
    if let Err(error) = render::main().await {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
}
