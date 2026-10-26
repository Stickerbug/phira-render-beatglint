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
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("tokio runtime");
    let _guard = runtime.enter();
    let result = render::main().await;
    if let Err(error) = result {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
    // macroquad/prpr may leave wakeable futures registered on the runtime.
    // Exit like the upstream Tauri launcher instead of dropping that runtime
    // after GL teardown, which can trigger a late segfault in headless mode.
    std::process::exit(0);
}
