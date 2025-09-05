use axum::{
    routing::{get, post},
    Json, Router, extract::Query, extract::State
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing_subscriber;
use derive_more::From;
use serde_json;
use time::{OffsetDateTime};
use std::{
    str::FromStr,
    error,
    fmt::{self, Debug, Display, Formatter},
    fs,
    ops::Not,
    time::Duration,
    thread,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use sqlx::{Column, Row, SqlitePool};
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, RwLock};
/// A submodule that handles save file parsing
use ck3_history_extractor::parser;
use parser::{SaveFileError};

/// A submodule for handling Steam integration
use ck3_history_extractor::sql_util::data_proc::data_proc;

/// An error a user has caused. Shame on him.
#[derive(From, Debug)]
enum UserError {
    /// The program is not running in a terminal
    NoTerminal,
    /// The file does not exist
    FileDoesNotExist,
    /// An error occurred during file handling
    FileError(SaveFileError),
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoTerminal => write!(f, "The program is not running in a terminal"),
            UserError::FileDoesNotExist => write!(f, "The file does not exist"),
            UserError::FileError(e) => write!(f, "An error occurred during file handling: {}", e),
        }
    }
}

impl error::Error for UserError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            UserError::FileError(e) => Some(e),
            _ => None,
        }
    }
}

type SaveStates = Arc<RwLock<HashMap<String, SaveState>>>;

enum SaveState {
    DataLoading,
    Prepared
}

enum ServerState {
    Initializing,
    Running,
    Stopped
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub save_path: String,
    pub game_path: String,
    pub mod_paths: Vec<String>,
    pub language: String,
    // 其他配置项
}

#[derive(Debug)]
struct SaveJob {
    save_path: String,
    game_path: String,
    mod_paths: Option<Vec<String>>
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<SaveJob>,
    save_states: SaveStates,
}

pub async fn load_config() -> Result<Config, Box<dyn error::Error>> {
    let content = fs::read_to_string("./config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// 异步初始化函数
/// 检查并创建SQLite数据库，查询metadata表并填充SaveStates，最后将ServerState设置为Running
async fn initialize_server(
    pool: &SqlitePool,
    save_states: SaveStates,
    server_state: Arc<RwLock<ServerState>>,
) -> Result<(), Box<dyn error::Error>> {


    let init_sql = include_str!("../sql_util/init.sql");
    match sqlx::query(init_sql)
        .execute(pool)
        .await {
        Ok(_) => tracing::info!("数据库构建完成"),
        Err(e) => {
            tracing::error!("SQL脚本执行失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 查询game_metadata表中的save_file_name字段
    let rows = sqlx::query("SELECT save_file_name FROM game_metadata")
        .fetch_all(pool)
        .await?;
    
    // 填充SaveStates
    {
        let mut states = save_states.write().await;
        for row in rows {
            let filename: String = row.get("save_file_name");
            states.insert(filename.clone(), SaveState::Prepared);
            tracing::info!("已加载存档文件状态: {}", &filename);
        }
        tracing::info!("总共加载了 {} 个存档文件", states.len());
    }
    
    // 将ServerState更新为Running
    {
        let mut state = server_state.write().await;
        *state = ServerState::Running;
        tracing::info!("服务器状态已更新为Running");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // 日志初始化
    tracing_subscriber::fmt::init();

    // 初始化服务器状态
    let (tx, mut rx) = mpsc::channel::<SaveJob>(8);
    let save_states: SaveStates = Arc::new(RwLock::new(HashMap::new()));
    let server_state = Arc::new(RwLock::new(ServerState::Initializing));

    // 加载配置
    let config = load_config().await.unwrap();
    let opts = SqliteConnectOptions::from_str(&config.database_url).unwrap().foreign_keys(false);
    let pool = SqlitePoolOptions::new().connect_with(opts).await.unwrap();

    // 执行初始化
    let save_states_clone = save_states.clone();
    let server_state_clone = server_state.clone();
    
    match initialize_server(&pool, save_states_clone, server_state_clone).await {
        Ok(_) => {
            tracing::info!("服务器初始化成功");
        }
        Err(e) => {
            tracing::error!("服务器初始化失败: {}", e);
            return;
        }
    }
    let ss_work = save_states.clone();
    thread::spawn(move || {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async move {
            while let Some(job) = rx.recv().await {
                // 这里是后台 worker 的异步逻辑：
                tracing::info!("收到存档处理任务: {}", job.save_path.clone());
                let save_path = std::path::PathBuf::from(job.save_path.clone());
                let mut ss = ss_work.as_ref().write().await;
                ss.insert(save_path.file_name().unwrap().to_string_lossy().to_string(), SaveState::DataLoading);

                // 加载配置
                let config = load_config().await.unwrap();

                // 连接到数据库
                let opts = SqliteConnectOptions::from_str(&config.database_url).unwrap().foreign_keys(false);
                let pool = SqlitePoolOptions::new().connect_with(opts).await.unwrap();

                // 转换mod_paths为PathBuf
                let mod_paths: Vec<std::path::PathBuf> = job
                    .mod_paths
                    .as_ref()
                    .map(|v| v.iter().map(std::path::PathBuf::from).collect())
                    .unwrap_or_default();
                // 使用data_proc处理存档文件
                match data_proc(&save_path, &mod_paths.clone(), "chinese", &pool).await {
                    Ok(_) => {
                        tracing::info!("存档文件处理成功: {}", job.save_path);

                    }
                    Err(e) => {
                        tracing::error!("存档文件处理失败: {}", e);
                    }
                }
                let mut ss = ss_work.as_ref().write().await;
                ss.insert(save_path.file_name().unwrap().to_string_lossy().to_string(), SaveState::Prepared);
                tracing::info!("存档文件状态更新为Prepared: {}", job.save_path);
            }
        });
    });

    // 构建路由
    let ss = save_states.clone();
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/v2/save_update", post(save_handler))
        .route("/v2/session_status", get(status_handler))
        .route("/v2/query", post(query_handler))
        .route("/v2/schema", get(schema_handler))
        //.route("/v2/save_lists", get())
        .with_state(AppState{ tx, save_states: ss });

    // 绑定地址
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3047").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Welcome to ck3 rust server!"
}


#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponsesReceivePayload {
    input: String,
    previous_id: Option<String>,
    screenshot: Option<String>,
    file_address: String,
    stream: bool
}

#[derive(Debug, Deserialize, Serialize)]
struct SaveUpdateReceivePayload {
    save_path: String,
    game_data: String,
    mod_paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponsesReturnPayload {
    created_at: OffsetDateTime,
    id: String,
    output: String,
    code: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct MyQuery {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct QueryRequest {
    sql: String,
}


async fn responses_handler(Json(payload) : Json<ResponsesReceivePayload>) -> Json<ResponsesReturnPayload> {
    // TODO: Responses handling logic
    let now_time = OffsetDateTime::now_utc();
    let response = ResponsesReturnPayload {
        created_at: now_time,
        id: "some_unique_id".to_string(), // TODO: Get a unique ID from the actual logic
        output: format!("Received input: {}", payload.input), // TODO: Return the actual output
        code: 200,
    };
    Json(response)
}

async fn status_handler(Query(params): Query<MyQuery>) -> Json<ApiResponse> {
    // TODO: Complete the status handler logic
    let response = ApiResponse {
        code: 200,
        message: "Server is running".to_string(),
    };
    Json(response)
}


async fn save_handler(State(state): State<AppState>, Json(payload): Json<SaveUpdateReceivePayload>) -> Json<ApiResponse> {
    // 验证存档文件路径
    let save_path = std::path::Path::new(&payload.save_path);
    let filename = match save_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => {
            // 处理无文件名情况
            return Json(ApiResponse {
                code: 400,
                message: "路径无文件名".to_string(),
            });
        }
    };
    let ss = state.save_states.read().await;
    if ss.contains_key(&filename.to_string()){
        if let Some(state) = ss.get(&filename.to_string()) {
            return match state {
                SaveState::DataLoading => {
                    Json(ApiResponse {
                        code: 409,
                        message: format!("存档文件正在处理中: {}", payload.save_path),
                    })
                }
                SaveState::Prepared => {
                    Json(ApiResponse {
                        code: 200,
                        message: format!("存档文件已存在且已处理: {}", payload.save_path),
                    })
                }
            }
        }
    }
    if !save_path.exists() {
        return Json(ApiResponse {
            code: 400,
            message: format!("存档文件不存在: {}", payload.save_path),
        });
    }
    let game_data = std::path::Path::new(&payload.game_data);
    if !game_data.exists() {
        return Json(ApiResponse {
            code: 400,
            message: format!("游戏数据不存在: {}", payload.game_data),
        });
    }
    let mut mod_paths: Vec<std::path::PathBuf> = Vec::new();
    // 转换mod_paths为PathBuf
    payload.mod_paths.as_ref().map(|m| {
        for p in m {
            mod_paths.push(std::path::PathBuf::from(p));
        }
    });
    for p in &mod_paths {
        if !p.exists() {
            return Json(ApiResponse {
                code: 400,
                message: format!("mod路径不存在: {:?}", p),
            });
        }
    }
    if let Err(e) = state.tx.send(
        SaveJob {
            save_path: payload.save_path.clone(),
            game_path: payload.game_data.clone(),
            mod_paths: payload.mod_paths.clone(),
        }
    ).await {
        tracing::error!("发送存档处理任务失败: {}", e);
        return Json(ApiResponse { code: 500, message: "派发任务失败（通道已关闭）".into() });
    };
    Json(ApiResponse {
        code: 202,
        message: "Save handler executed".to_string(),
    })
}

/// 验证SQL查询语句是否安全（只允许SELECT查询）
fn is_safe_sql_query(sql: &str) -> bool {
    let sql = sql.trim().to_lowercase();
    
    // 检查是否以SELECT开头
    if !sql.starts_with("select ") {
        return false;
    }
    
    // 禁止的危险关键词
    let dangerous_keywords = [
        "insert", "update", "delete", "drop", "create", "alter", "truncate",
        "replace", "merge", "upsert", "attach", "detach", "pragma"
    ];
    
    for keyword in &dangerous_keywords {
        if sql.contains(keyword) {
            return false;
        }
    }
    
    // 禁止分号（防止多语句执行）
    if sql.contains(';') {
        return false;
    }
    
    // 禁止注释（防止绕过检查）
    if sql.contains("--") || sql.contains("/*") || sql.contains("*/") {
        return false;
    }
    
    true
}
fn sqlite_row_to_json(row: &SqliteRow) -> Value {
    let mut obj = serde_json::Map::new();

    for (i, col) in row.columns().iter().enumerate() {
        let name = col.name().to_string();

        // 按类型“试取”，谁先成功用谁。注意顺序：先数值，再文本，最后二进制。
        let v = if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
            v.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
            v.and_then(|f| serde_json::Number::from_f64(f).map(Value::Number))
                .unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
            v.map(Value::String).unwrap_or(Value::Null)
        } else if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(i) {
            match v {
                Some(bytes) => Value::String(format!("base64:{}", base64::encode(bytes))),
                None => Value::Null,
            }
        } else {
            Value::Null
        };

        obj.insert(name, v);
    }

    Value::Object(obj)
}

async fn query_handler(State(state): State<AppState>, Json(payload): Json<QueryRequest>) -> Json<Value> {
    // 验证SQL查询语句
    if !is_safe_sql_query(&payload.sql) {
        return Json(json!( {
            "code": 400,
            "message": "不安全的SQL查询语句，只允许SELECT查询".to_string(),
        }));
    }

    // 加载配置
    let config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("加载配置失败: {}", e);
            return Json(json!( {
                "code": 500,
                "message": format!("配置加载失败: {}", e),
            }));
        }
    };

    // 连接到数据库
    let opts = SqliteConnectOptions::from_str(&config.database_url).unwrap().foreign_keys(false);
    let pool = match SqlitePoolOptions::new().connect_with(opts).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("数据库连接失败: {}", e);
            return Json(json!( {
                "code": 500,
                "message": format!("数据库连接失败: {}", e),
            }));
        }
    };

    // 执行查询
    match sqlx::query(&payload.sql).fetch_all(&pool).await {
        Ok(rows) => {
            // 关闭数据库连接
            let rows_json: Vec<Value> = rows.iter().map(sqlite_row_to_json).collect();

            Json(json!({
                "code": 200,
                "message": format!("查询执行成功，返回 {} 行数据", rows_json.len()),
                "data": {
                    "count": rows_json.len(),
                    "rows": rows_json
                }
            }))
        }
        Err(e) => {
            // 关闭数据库连接

            tracing::error!("SQL查询执行失败: {}", e);
            Json(json!({
                "code": 500,
                "message": format!("SQL查询执行失败: {}", e),
                "data": {
                    "count": 0,
                    "rows": []
                }
            }))
        }
    }
}

async fn schema_handler() -> Json<ApiResponse> {
    // TODO: Complete the schema handler logic
    let response = ApiResponse {
        code: 200,
        message: "Schema handler executed".to_string(),
    };
    Json(response)
}

