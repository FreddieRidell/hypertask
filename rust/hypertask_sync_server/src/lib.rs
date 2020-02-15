extern crate hypertask_engine;

mod config;

use crate::config::SyncServerConfig;
use actix_web::{get, post};
use actix_web::{web, App, HttpRequest, HttpServer};
use chrono::prelude::*;
use hypertask_config_file_opener::{ConfigFileGetter, ConfigFileOpener};
use hypertask_engine::prelude::*;
use hypertask_task_io_operations::{delete_task, get_input_tasks, get_task, put_task};
use std::collections::HashMap;
use std::rc::Rc;
use time::Duration;

type TaskHashes = HashMap<Rc<Id>, u64>;

#[post("/task/{id}")]
fn compare_tasks(
    config_data: web::Data<SyncServerConfig>,
    path: web::Path<String>,
    client_task_input: web::Json<Option<Task>>,
    req: HttpRequest,
) -> actix_web::Result<web::Json<Option<Task>>> {
    if let Some(Ok(auth_header)) = req.headers().get("Authorization").map(|x| x.to_str()) {
        if format!("hypertask {}", &config_data.sync_secret) != auth_header {
            return Err(actix_web::error::ErrorUnauthorized(
                "invalid sync_secret provided",
            ));
        }
    }

    let id = Id(path.to_string());
    let config: &SyncServerConfig = config_data.get_ref();

    let server_task: Option<Task> = get_task(config, &id).expect("could not open task");

    let client_task: Option<Task> = client_task_input.into_inner();

    let resolved_task: Option<Task> =
        Task::resolve_task_conflict(&(Utc::now() - Duration::days(30)), server_task, client_task)
            .expect("tasks did not have the same id");

    match &resolved_task {
        Some(task) => put_task(config, &task).expect("could not save task"),
        None => delete_task(config, &id).expect("could not delete task"),
    };

    Ok(web::Json(resolved_task))
}

#[get("/hashes")]
fn get_hashes(
    config_data: web::Data<SyncServerConfig>,
    req: HttpRequest,
) -> actix_web::Result<web::Json<TaskHashes>> {
    if let Some(Ok(auth_header)) = req.headers().get("Authorization").map(|x| x.to_str()) {
        if format!("hypertask {}", &config_data.sync_secret) != auth_header {
            return Err(actix_web::error::ErrorUnauthorized(
                "invalid sync_secret provided",
            ));
        }
    }

    let mut task_hashes = TaskHashes::new();
    let config: &SyncServerConfig = config_data.get_ref();

    let input_tasks: HashMap<Rc<Id>, Rc<Task>> =
        get_input_tasks(config).expect("could not get tasks");

    for (id, task) in input_tasks.iter() {
        task_hashes.insert(id.clone(), task.calculate_hash());
    }

    Ok(web::Json(task_hashes))
}

fn get_config_object() -> HyperTaskResult<SyncServerConfig> {
    let mut config_file_opener = ConfigFileOpener::new("sync-server.toml")?;
    let config_file_getter: ConfigFileGetter<SyncServerConfig> = config_file_opener.parse()?;
    Ok(config_file_getter.get_config().clone())
}

pub fn start() -> HyperTaskResult<()> {
    let sync_server_config = get_config_object()?;

    println!(
        "started syncing server for dir `{}` @ http://{}:{}",
        sync_server_config
            .task_state_dir
            .to_str()
            .expect("could not read task_state_dir"),
        sync_server_config.hostname,
        sync_server_config.port,
    );

    HttpServer::new(|| {
        let config = get_config_object().expect("could not load config");

        App::new()
            .data(config)
            .service(get_hashes)
            .service(compare_tasks)
    })
    .bind((
        sync_server_config.hostname.as_str(),
        sync_server_config.port,
    ))
    .expect("could not create server")
    .run()
    .expect("could not run server");

    Ok(())
}
