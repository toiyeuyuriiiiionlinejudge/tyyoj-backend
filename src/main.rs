use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::{header, HeaderName, HeaderValue, Method, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bollard::query_parameters::ListContainersOptions;
use bollard::Docker;
use lettre::message::header::ContentType;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tyyoj_contracts::{JudgeMode, JudgeTask, SubmissionLanguage, SubmissionVerdict};
use uuid::Uuid;

// Global Application State
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
}

// In-Memory Token Bucket Rate Limiter
pub struct RateLimiter {
    clients: HashMap<String, (u32, Instant)>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            clients: HashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check_and_record(&mut self, client_ip: &str) -> bool {
        let now = Instant::now();
        let entry = self.clients.entry(client_ip.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > self.window {
            entry.0 = 1;
            entry.1 = now;
            true
        } else if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

// Database Models (PostgreSQL 17)
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProblemRecord {
    pub id: Uuid,
    pub code: String,
    pub title: String,
    pub difficulty: String,
    pub statement_markdown: String,
    pub time_limit_ms: i64,
    pub memory_limit_mb: i64,
    pub judge_mode: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ContestRecord {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub rule_type: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrganizationRecord {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub member_count: i32,
    pub contests_count: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlogRecord {
    pub id: Uuid,
    pub title: String,
    pub author_name: String,
    pub content_markdown: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WikiRecord {
    pub id: Uuid,
    pub title: String,
    pub category: String,
    pub content_markdown: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SubmissionRecord {
    pub id: Uuid,
    pub problem_code: String,
    pub language: String,
    pub verdict: String,
    pub time_consumed_ms: i64,
    pub memory_consumed_kb: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionPayload {
    pub problem_code: String,
    pub language: String,
    pub source_code: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RouteCatalogEntry {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub description: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tyyoj_user:tyyoj_dev_password@127.0.0.1:5432/tyyoj".into());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".into());

    tracing::info!("Connecting to PostgreSQL database at {}", db_url);
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connecting to Redis at {}", redis_url);
    let redis = redis::Client::open(redis_url).expect("Failed to create Redis client");

    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(100, 60))); // 100 reqs / min per client

    let state = AppState {
        db,
        redis,
        rate_limiter,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/verdicts", get(list_verdicts))
        .route("/api/v1/problems", get(list_problems))
        .route("/api/v1/problems/{id_or_code}", get(get_problem))
        .route("/api/v1/contests", get(list_contests))
        .route("/api/v1/organizations", get(list_organizations))
        .route("/api/v1/blogs", get(list_blogs))
        .route("/api/v1/wikis", get(list_wikis))
        .route("/api/v1/submissions", get(list_submissions).post(create_submission))
        .route("/api/v1/auth/register", post(register_user))
        .route("/api/v1/auth/login", post(login_user))
        .route("/api/v1/admin/judgers", get(list_admin_judgers))
        .route("/api/v1/admin/routes", get(list_admin_routes))
        .layer(cors)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("tyyoj-backend running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// OWASP Security Headers Middleware
async fn security_headers_middleware(req: Request<axum::body::Body>, next: Next) -> Response<axum::body::Body> {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();

    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    res
}

// Rate Limiting Middleware
async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    let allowed = {
        let mut limiter = state.rate_limiter.lock().unwrap();
        limiter.check_and_record(&client_ip)
    };

    if !allowed {
        tracing::warn!("Rate limit exceeded for IP {}", client_ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

// Handler: Health Check
async fn health_check(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    if db_ok {
        Ok("OK")
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// Handler: Verdicts
async fn list_verdicts() -> Json<Vec<SubmissionVerdict>> {
    Json(vec![
        SubmissionVerdict::Pending,
        SubmissionVerdict::Compiling,
        SubmissionVerdict::Running,
        SubmissionVerdict::Accepted,
        SubmissionVerdict::WrongAnswer,
        SubmissionVerdict::TimeLimitExceeded,
        SubmissionVerdict::MemoryLimitExceeded,
        SubmissionVerdict::RuntimeError,
        SubmissionVerdict::CompilationError,
        SubmissionVerdict::IdlenessLimitExceeded,
    ])
}

// Handler: List Problems (PostgreSQL 17)
async fn list_problems(State(state): State<AppState>) -> Result<Json<Vec<ProblemRecord>>, StatusCode> {
    let problems = sqlx::query_as::<_, ProblemRecord>(
        "SELECT id, code, title, difficulty, statement_markdown, time_limit_ms, memory_limit_mb, judge_mode FROM problems ORDER BY code ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch problems from database: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(problems))
}

// Handler: Get Problem Detail (PostgreSQL 17)
async fn get_problem(
    State(state): State<AppState>,
    Path(id_or_code): Path<String>,
) -> Result<Json<ProblemRecord>, StatusCode> {
    let problem = if let Ok(uuid) = Uuid::parse_str(&id_or_code) {
        sqlx::query_as::<_, ProblemRecord>(
            "SELECT id, code, title, difficulty, statement_markdown, time_limit_ms, memory_limit_mb, judge_mode FROM problems WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(&state.db)
        .await
    } else {
        sqlx::query_as::<_, ProblemRecord>(
            "SELECT id, code, title, difficulty, statement_markdown, time_limit_ms, memory_limit_mb, judge_mode FROM problems WHERE code = $1"
        )
        .bind(&id_or_code)
        .fetch_optional(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    problem.map(Json).ok_or(StatusCode::NOT_FOUND)
}

// Handler: List Contests (PostgreSQL 17)
async fn list_contests(State(state): State<AppState>) -> Result<Json<Vec<ContestRecord>>, StatusCode> {
    let contests = sqlx::query_as::<_, ContestRecord>(
        "SELECT id, title, description, start_time, end_time, rule_type FROM contests ORDER BY start_time DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(contests))
}

// Handler: List Organizations (PostgreSQL 17)
async fn list_organizations(State(state): State<AppState>) -> Result<Json<Vec<OrganizationRecord>>, StatusCode> {
    let orgs = sqlx::query_as::<_, OrganizationRecord>(
        "SELECT id, name, description, member_count, contests_count FROM organizations ORDER BY member_count DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(orgs))
}

// Handler: List Blogs (PostgreSQL 17)
async fn list_blogs(State(state): State<AppState>) -> Result<Json<Vec<BlogRecord>>, StatusCode> {
    let blogs = sqlx::query_as::<_, BlogRecord>(
        "SELECT id, title, author_name, content_markdown, created_at FROM blogs ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(blogs))
}

// Handler: List Wikis (PostgreSQL 17)
async fn list_wikis(State(state): State<AppState>) -> Result<Json<Vec<WikiRecord>>, StatusCode> {
    let wikis = sqlx::query_as::<_, WikiRecord>(
        "SELECT id, title, category, content_markdown, updated_at FROM wikis ORDER BY category ASC, title ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(wikis))
}

// Handler: List Submissions (PostgreSQL 17)
async fn list_submissions(State(state): State<AppState>) -> Result<Json<Vec<SubmissionRecord>>, StatusCode> {
    let subs = sqlx::query_as::<_, SubmissionRecord>(
        "SELECT s.id, p.code as problem_code, s.language, s.verdict, s.time_consumed_ms, s.memory_consumed_kb, s.created_at
         FROM submissions s
         JOIN problems p ON s.problem_id = p.id
         ORDER BY s.created_at DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch submissions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(subs))
}

// Handler: Create Submission (PostgreSQL 17 + Redis Queue)
async fn create_submission(
    State(state): State<AppState>,
    Json(payload): Json<CreateSubmissionPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    let problem = sqlx::query_as::<_, ProblemRecord>(
        "SELECT id, code, title, difficulty, statement_markdown, time_limit_ms, memory_limit_mb, judge_mode FROM problems WHERE code = $1"
    )
    .bind(&payload.problem_code)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if payload.source_code.len() > 65536 || payload.source_code.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let submission_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, problem_id, language, source_code, verdict, time_consumed_ms, memory_consumed_kb)
         VALUES ($1, $2, $3, $4, 'Pending', 0, 0)"
    )
    .bind(submission_id)
    .bind(problem.id)
    .bind(&payload.language)
    .bind(&payload.source_code)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert submission: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let task = JudgeTask {
        submission_id,
        problem_id: problem.id,
        language: match payload.language.to_lowercase().as_str() {
            "pascal" => SubmissionLanguage::Pascal,
            "python3" | "python" => SubmissionLanguage::Python3,
            _ => SubmissionLanguage::Cpp20,
        },
        judge_mode: match problem.judge_mode.as_str() {
            "CustomChecker" => JudgeMode::CustomChecker,
            "Interactive" => JudgeMode::Interactive,
            _ => JudgeMode::Standard,
        },
        source_code: payload.source_code,
        time_limit_ms: problem.time_limit_ms as u64,
        memory_limit_mb: problem.memory_limit_mb as u64,
    };

    let serialized_task = serde_json::to_string(&task).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut redis_conn = state.redis.get_multiplexed_async_connection().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _: () = redis_conn
        .lpush("tyyoj:queue:submissions", serialized_task)
        .await
        .map_err(|e| {
            tracing::error!("Failed to push job to Redis: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Submission {} queued successfully in Redis", submission_id);
    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({ "submission_id": submission_id, "status": "Pending" }))))
}

// Handler: Register User (Argon2id + SMTP Mailpit)
async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.username.len() < 3 || payload.password.len() < 8 || !payload.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid username, password or email format".into()));
    }

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let user_id = Uuid::new_v4();
    let verification_token = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, is_verified, verification_token)
         VALUES ($1, $2, $3, $4, 'user', FALSE, $5)"
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&verification_token)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Registration failed: {}", e);
        (StatusCode::CONFLICT, "Username or email already exists".into())
    })?;

    let email = Message::builder()
        .from("no-reply@tyyoj.local".parse().unwrap())
        .to(payload.email.parse().unwrap())
        .subject("Kích hoạt tài khoản Toi Yêu Yuri Online Judge")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Chào mừng {} đến với Toi Yêu Yuri Online Judge!\nMã kích hoạt tài khoản của bạn: {}",
            payload.username, verification_token
        ))
        .unwrap();

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
            .port(1025)
            .tls(Tls::None)
            .build();

    if let Err(e) = mailer.send(email).await {
        tracing::warn!("Failed to dispatch verification email via Mailpit: {}", e);
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "user_id": user_id,
        "message": "User registered successfully. Verification email dispatched."
    }))))
}

// Handler: Login User (Argon2id Verification)
async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = sqlx::query_as::<_, UserRecord>(
        "SELECT id, username, email, password_hash, role FROM users WHERE username = $1 OR email = $1"
    )
    .bind(&payload.username_or_email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database query error".into()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid username or password".into()))?;

    let parsed_hash = argon2::password_hash::phc::PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash stored".into()))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username or password".into()))?;

    let token = format!("tyyoj_token_{}_{}", user.id, Uuid::new_v4());
    Ok(Json(serde_json::json!({
        "user_id": user.id,
        "username": user.username,
        "role": user.role,
        "access_token": token,
        "token_type": "Bearer"
    })))
}

// Handler: Admin Judgers (queries Podman Engine Socket API)
async fn list_admin_judgers() -> Result<impl IntoResponse, StatusCode> {
    let docker = Docker::connect_with_unix(
        "/run/podman/podman.sock",
        120,
        bollard::API_DEFAULT_VERSION,
    )
    .map_err(|e| {
        tracing::error!("Podman socket connect error: {}", e);
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap_or_default();

    let judgers: Vec<_> = containers
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id.unwrap_or_default(),
                "name": c.names.unwrap_or_default().first().cloned().unwrap_or_default(),
                "status": c.status.unwrap_or_default(),
                "state": format!("{:?}", c.state),
                "image": c.image.unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(judgers))
}

// Handler: Admin Routes Catalog
async fn list_admin_routes() -> Json<Vec<RouteCatalogEntry>> {
    Json(vec![
        RouteCatalogEntry { method: "GET".into(), path: "/health".into(), handler: "health_check".into(), description: "Kiểm tra tình trạng cơ sở dữ liệu và Redis".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/verdicts".into(), handler: "list_verdicts".into(), description: "Danh sách 10 trạng thái kết quả chấm".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/problems".into(), handler: "list_problems".into(), description: "Truy vấn danh sách bài tập từ PostgreSQL".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/problems/{id_or_code}".into(), handler: "get_problem".into(), description: "Chi tiết bài tập và công thức toán KaTeX".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/contests".into(), handler: "list_contests".into(), description: "Danh sách kỳ thi thuật toán".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/organizations".into(), handler: "list_organizations".into(), description: "Danh sách tổ chức và câu lạc bộ".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/blogs".into(), handler: "list_blogs".into(), description: "Bài viết và thông báo hệ thống".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/wikis".into(), handler: "list_wikis".into(), description: "Cẩm nang tra cứu và quy chuẩn biên dịch".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/submissions".into(), handler: "list_submissions".into(), description: "Tra cứu bài nộp gần đây từ PostgreSQL".into() },
        RouteCatalogEntry { method: "POST".into(), path: "/api/v1/submissions".into(), handler: "create_submission".into(), description: "Nộp bài, lưu DB và xếp hàng Redis queue".into() },
        RouteCatalogEntry { method: "POST".into(), path: "/api/v1/auth/register".into(), handler: "register_user".into(), description: "Đăng ký tài khoản với băm mật khẩu Argon2id".into() },
        RouteCatalogEntry { method: "POST".into(), path: "/api/v1/auth/login".into(), handler: "login_user".into(), description: "Đăng nhập và cấp token xác thực".into() },
        RouteCatalogEntry { method: "GET".into(), path: "/api/v1/admin/judgers".into(), handler: "list_admin_judgers".into(), description: "Tra cứu trạng thái container qua Podman Socket API".into() },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, 60);
        assert!(limiter.check_and_record("192.168.1.1"));
        assert!(limiter.check_and_record("192.168.1.1"));
        assert!(!limiter.check_and_record("192.168.1.1")); // Rate limit exceeded!
        assert!(limiter.check_and_record("192.168.1.2")); // Other IP allowed
    }

    #[test]
    fn test_argon2id_hashing() {
        let password = b"secure_contestant_password_2026";
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password).unwrap().to_string();
        let parsed = argon2::password_hash::phc::PasswordHash::new(&hash).unwrap();
        assert!(argon2.verify_password(password, &parsed).is_ok());
    }
}
