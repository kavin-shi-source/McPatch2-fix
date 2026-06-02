use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::app_path::AppPath;
use crate::config::auth_config::AuthConfig;
use crate::config::Config;
use crate::web::file_status::FileStatus;
use crate::web::log::Console;
use crate::web::task_executor::LongTimeExecutor;

#[derive(Clone)]
pub struct WebState {
    pub apppath: AppPath,
    pub config: Config,
    pub auth: AuthConfig,
    pub te: Arc<Mutex<LongTimeExecutor>>,
    pub status: Arc<Mutex<FileStatus>>,
    pub console: Console,
    pub login_attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl WebState {
    pub fn new(apppath: AppPath, config: Config, auth: AuthConfig) -> Self {
        Self {
            apppath,
            config,
            auth,
            te: Arc::new(Mutex::new(LongTimeExecutor::new())),
            status: Arc::new(Mutex::new(FileStatus::new(apppath, config))),
            console: Console::new_webui(),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
