use subprocess::{Popen, PopenConfig};
use warp::{
    http::{Response, StatusCode},
    Filter,
};
use chrono::Utc;

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

enum LogLevel {
    INFO,
    ERROR,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LogLevel::*;
        let s: String = match &self {
            INFO => "INFO".into(),
            ERROR => "ERROR".into()
        };
        write!(f, "{s}")
    }
}

fn log(level: LogLevel, msg: impl std::fmt::Display) {
    let time = Utc::now();
    println!("{time} - {level} - {msg}");
}

fn get_script_argv(script: Script) -> &'static [&'static str] {
    match (script, getenv("HOOK_ENV")) {
        (_, x) if x == String::from("dev") => &["echo", "foo"],
        (Script::Update, _) =>  &["./update.sh", MARAIN_SOURCE_DIR],
        (Script::Relaunch, _) => &["./relaunch.sh", MARAIN_SOURCE_DIR],
    }
}

fn update_and_relaunch() -> HookResult {
    log(LogLevel::INFO, "Update hook called");
    let Ok(mut update_proc) = Popen::create(get_script_argv(Script::Update), PopenConfig::default()) else {
        log(LogLevel::ERROR, "Failed to create update subprocess");
        return HookResult::ServerError;
    };

    log(LogLevel::INFO, "Waiting for update process");
    match update_proc.wait() {
        Ok(subprocess::ExitStatus::Exited(0)) => {
            log(LogLevel::INFO, "Update process successful.");
            let update_result = HookResult::Ok;

            let Ok(mut relaunch_proc) = Popen::create(get_script_argv(Script::Relaunch), PopenConfig::default())
            else {
                log(LogLevel::ERROR, "Failed to create relaunch subprocess");
                return HookResult::ServerError;
            };
            log(LogLevel::INFO, "Relaunch subprocess created, detaching.");
            relaunch_proc.detach();
            update_result
        },
        Ok(fail) => {
            log(LogLevel::ERROR, format!("Update failed with exit code: {fail:?}"));
            HookResult::UpdateFailed
        },
        Err(e) => {
            log(LogLevel::ERROR, format!("Update subprocess wait failed with error: {e}"));
            HookResult::ServerError
        },
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
    log(LogLevel::INFO, "Hook server starting");
    warp::serve(update).run(([0, 0, 0, 0], 42069)).await;
}
