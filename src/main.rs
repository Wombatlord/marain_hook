use subprocess::{Popen, PopenConfig};
use warp::{
    http::{Response, StatusCode},
    Filter,
};

enum HookResult {
    Ok,
    UpdateFailed,
    ServerError,
}

fn getenv(name: &str) -> String {
    match std::env::var(name) {
        Ok(var) => var,
        _ => "".to_string(),
    }
}

enum Script {
    Update,
    Relaunch
}

const MARAIN_SOURCE_DIR: &str = "/var/www/marain";

fn get_script_argv(script: Script) -> &'static [&'static str] {
    match (script, getenv("HOOK_ENV")) {
        (_, x) if x == String::from("dev") => &["echo", "foo"],
        (Script::Update, _) =>  &["./update.sh", MARAIN_SOURCE_DIR],
        (Script::Relaunch, _) => &["./relaunch.sh", MARAIN_SOURCE_DIR],
    }
}

fn update_and_relaunch() -> HookResult {
    let Ok(mut update_proc) = Popen::create(get_script_argv(Script::Update), PopenConfig::default()) else {
        return HookResult::ServerError;
    };

    match update_proc.wait() {
        Ok(subprocess::ExitStatus::Exited(0)) => {
            let update_result = HookResult::Ok;

            let Ok(mut relaunch_proc) = Popen::create(get_script_argv(Script::Relaunch), PopenConfig::default())
            else {
                return HookResult::ServerError;
            };
            relaunch_proc.detach();
            update_result
        },
        Ok(_fail) => HookResult::UpdateFailed,
        Err(_) => HookResult::ServerError,
    }
}

#[tokio::main]
async fn main() {
    use HookResult::*;

    let update = warp::post()
        .and(warp::path::end())
        .map(|| {
            match update_and_relaunch() {
                Ok => Response::builder()
                    .status(StatusCode::OK)
                    .body("".to_string()),
                UpdateFailed => Response::builder()
                    .status(StatusCode::FAILED_DEPENDENCY)
                    .body("update failed\n".into()),
                ServerError => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("".to_string()),
            }
        });

    warp::serve(update).run(([0, 0, 0, 0], 42069)).await;
}
