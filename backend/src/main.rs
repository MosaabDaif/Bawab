use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// ─── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    jwt_secret: String,
}

// ─── Database models ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Scan {
    id: i64,
    student_id: String,
    action: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Admin {
    id: i64,
    username: String,
    password_hash: String,
}

// ─── Request / response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ScanRequest {
    student_id: String,
}

#[derive(Debug, Serialize)]
struct ScanResponse {
    action: String,
    student_id: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

// ─── JWT claims ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// ─── Analytics types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct InsideRecord {
    student_id: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct DailyCount {
    day: String,
    count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct HourCount {
    hour: String,
    count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct WeekdayAvg {
    weekday: String,
    avg_visitors: f64,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

type ApiError = (StatusCode, Json<Value>);
type ApiResult<T> = Result<T, ApiError>;

fn db_err(e: sqlx::Error) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    )
}

// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let opts = database_url
        .parse::<SqliteConnectOptions>()
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("Failed to connect to SQLite database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    seed_admin(&pool).await;

    let state = AppState { pool, jwt_secret };

    // CORS: permissive for development; tighten for production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Protected routes (require JWT)
    let protected = Router::new()
        .route("/inside", get(get_inside))
        .route("/log", get(get_log))
        .route("/analytics/daily", get(analytics_daily))
        .route("/analytics/peak", get(analytics_peak))
        .route("/analytics/weekday", get(analytics_weekday))
        .route("/export/csv", get(export_csv))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    let app = Router::new()
        .route("/scan", post(post_scan))
        .route("/auth/login", post(post_login))
        .merge(protected)
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ─── Database seeding ─────────────────────────────────────────────────────────

async fn seed_admin(pool: &SqlitePool) {
    let admin_password =
        std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "library2024".to_string());

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins")
        .fetch_one(pool)
        .await
        .expect("Failed to query admins table");

    if count == 0 {
        let password_hash =
            hash(&admin_password, DEFAULT_COST).expect("Failed to hash admin password");

        sqlx::query("INSERT INTO admins (username, password_hash) VALUES (?, ?)")
            .bind("admin")
            .bind(&password_hash)
            .execute(pool)
            .await
            .expect("Failed to seed admin user");

        println!("Seeded admin user  →  username: admin  |  password: {}", admin_password);
    }
}

// ─── Auth middleware ──────────────────────────────────────────────────────────

async fn require_auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(t) => match decode::<Claims>(
            t,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(_) => Ok(next.run(req).await),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired token" })),
            )),
        },
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Missing Authorization header" })),
        )),
    }
}

// ─── POST /scan ───────────────────────────────────────────────────────────────

async fn post_scan(
    State(state): State<AppState>,
    Json(body): Json<ScanRequest>,
) -> ApiResult<Json<ScanResponse>> {
    let student_id = body.student_id.trim().to_string();

    // Validate: 4–12 numeric digits
    if student_id.len() < 4
        || student_id.len() > 12
        || !student_id.chars().all(|c| c.is_ascii_digit())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Student ID must be 4–12 numeric digits" })),
        ));
    }

    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // Fetch last scan for this student
    let last_scan = sqlx::query_as::<_, Scan>(
        "SELECT id, student_id, action, timestamp \
         FROM scans WHERE student_id = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(&student_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_err)?;

    // Debounce: ignore if the previous scan was within the last 2 seconds
    if let Some(ref scan) = last_scan {
        if let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(&scan.timestamp) {
            if now
                .signed_duration_since(last_dt.with_timezone(&Utc))
                .num_seconds()
                < 2
            {
                return Ok(Json(ScanResponse {
                    action: scan.action.clone(),
                    student_id,
                }));
            }
        }
    }

    // Toggle IN → OUT or OUT / none → IN
    let action = match &last_scan {
        Some(s) if s.action == "IN" => "OUT",
        _ => "IN",
    };

    sqlx::query(
        "INSERT INTO scans (student_id, action, timestamp) VALUES (?, ?, ?)",
    )
    .bind(&student_id)
    .bind(action)
    .bind(&now_str)
    .execute(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(ScanResponse {
        action: action.to_string(),
        student_id,
    }))
}

// ─── GET /inside ──────────────────────────────────────────────────────────────

async fn get_inside(State(state): State<AppState>) -> ApiResult<Json<Vec<InsideRecord>>> {
    // Students whose most recent scan was IN
    let records = sqlx::query_as::<_, InsideRecord>(
        r#"
        SELECT s.student_id, s.timestamp
        FROM scans s
        INNER JOIN (
            SELECT student_id, MAX(timestamp) AS last_ts
            FROM scans
            GROUP BY student_id
        ) latest
            ON s.student_id = latest.student_id
           AND s.timestamp  = latest.last_ts
        WHERE s.action = 'IN'
        ORDER BY s.timestamp DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(records))
}

// ─── GET /log ─────────────────────────────────────────────────────────────────

async fn get_log(State(state): State<AppState>) -> ApiResult<Json<Vec<Scan>>> {
    let scans = sqlx::query_as::<_, Scan>(
        "SELECT id, student_id, action, timestamp \
         FROM scans ORDER BY timestamp DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(scans))
}

// ─── GET /analytics/daily ────────────────────────────────────────────────────

async fn analytics_daily(State(state): State<AppState>) -> ApiResult<Json<Vec<DailyCount>>> {
    let counts = sqlx::query_as::<_, DailyCount>(
        r#"
        SELECT DATE(timestamp) AS day,
               COUNT(DISTINCT student_id) AS count
        FROM scans
        WHERE action = 'IN'
          AND timestamp >= datetime('now', '-7 days')
        GROUP BY day
        ORDER BY day
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(counts))
}

// ─── GET /analytics/peak ─────────────────────────────────────────────────────

async fn analytics_peak(State(state): State<AppState>) -> ApiResult<Json<Vec<HourCount>>> {
    let counts = sqlx::query_as::<_, HourCount>(
        r#"
        SELECT strftime('%H', timestamp) AS hour,
               COUNT(*) AS count
        FROM scans
        WHERE DATE(timestamp) = DATE('now')
        GROUP BY hour
        ORDER BY hour
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(counts))
}

// ─── GET /analytics/weekday ──────────────────────────────────────────────────

async fn analytics_weekday(State(state): State<AppState>) -> ApiResult<Json<Vec<WeekdayAvg>>> {
    // Average unique visitors per weekday (0=Sun … 6=Sat)
    let avgs = sqlx::query_as::<_, WeekdayAvg>(
        r#"
        WITH daily AS (
            SELECT DATE(timestamp)              AS day,
                   strftime('%w', timestamp)    AS weekday,
                   COUNT(DISTINCT student_id)   AS visitors
            FROM scans
            WHERE action = 'IN'
            GROUP BY day
        )
        SELECT weekday, AVG(visitors) AS avg_visitors
        FROM daily
        GROUP BY weekday
        ORDER BY CAST(weekday AS INTEGER)
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    Ok(Json(avgs))
}

// ─── POST /auth/login ────────────────────────────────────────────────────────

async fn post_login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let admin = sqlx::query_as::<_, Admin>(
        "SELECT id, username, password_hash FROM admins WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid credentials" })),
        )
    })?;

    let valid = verify(&body.password, &admin.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid credentials" })),
        ));
    }

    // JWT expires in 24 h
    let exp = (Utc::now().timestamp() + 86_400) as usize;
    let claims = Claims {
        sub: admin.username,
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(LoginResponse { token }))
}

// ─── GET /export/csv ─────────────────────────────────────────────────────────

async fn export_csv(State(state): State<AppState>) -> ApiResult<Response> {
    let scans = sqlx::query_as::<_, Scan>(
        "SELECT id, student_id, action, timestamp \
         FROM scans ORDER BY timestamp ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_err)?;

    let mut csv = String::from("id,student_id,action,timestamp\n");
    for s in scans {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            s.id, s.student_id, s.action, s.timestamp
        ));
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"library_scans.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .unwrap();

    Ok(response)
}
