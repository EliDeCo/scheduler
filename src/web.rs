use crate::structs::DisplaySchedule;
use axum::{Router, response::Html, routing::get};
use minijinja::{Environment, context, path_loader};
use std::sync::Arc;

pub async fn launch_webpage(all_schedules: Vec<DisplaySchedule>) {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    let env = Arc::new(env);

    let app = Router::new().route(
        "/display",
        get({
            let env = env.clone();
            move || {
                let env = env.clone();
                async move {
                    let tmpl = env.get_template("display.html").unwrap();
                    let rendered = tmpl.render(context! { all_schedules }).unwrap();
                    Html(rendered)
                }
            }
        }),
    );

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("127.0.0.1:7878")
        .await
        .unwrap();
    println!("Launching webpage");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
