mod capturing_writer;

use capturing_writer::CapturingWriter;
use sqlx::{migrate::MigrateDatabase as _, Sqlite, SqlitePool};
use std::env;
use std::process::{Command, Stdio};
use structopt::StructOpt;

const STATUS_ERR_SIGNAL: i32 = -1;
const STATUS_ERR_SPAWN: i32 = -2;

type BoxError = Box<dyn std::error::Error>;

include!(concat!(env!("OUT_DIR"), "/curl_opts.rs"));

#[derive(Debug)]
struct Request {
    method: String,
    url: String,
}

fn try_parse_request() -> Result<Request, BoxError> {
    let mut opts = Opts::from_args_safe()?;
    Ok(Request {
        method: opts.request.take().unwrap_or_else(|| "GET".into()),
        url: opts
            .url
            .take()
            .or_else(|| opts.url_arg.first().cloned())
            .ok_or("missing url")?,
    })
}

#[derive(Debug)]
struct CurlResult {
    exit_code: i32,
    output: String,
}

fn try_run_curl() -> Result<CurlResult, BoxError> {
    let mut child = Command::new("curl")
        .args(env::args_os().skip(1))
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child.stdout.take().expect("stdout is piped");
    let stdout = std::io::stdout();
    let mut stdout_capture = CapturingWriter::new(stdout.lock());
    let _ = std::io::copy(&mut child_stdout, &mut stdout_capture);

    let status = child.wait().expect("command wasn't running");
    let output = stdout_capture.into_string();

    Ok(CurlResult {
        exit_code: status.code().unwrap_or(STATUS_ERR_SIGNAL),
        output,
    })
}

async fn ensure_db() -> Result<SqlitePool, BoxError> {
    let database_path: String = if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("curl-history");
        std::fs::create_dir_all(&dir)?;
        dir.join("history.db")
            .to_str()
            .expect("config dir should be valid utf8 string")
            .into()
    } else {
        "history.db".into()
    };
    let database_url = format!("sqlite:{}", database_path);
    Sqlite::create_database(&database_url).await?;
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

async fn save_request(req: Request, res: CurlResult) -> Result<(), BoxError> {
    let pool = ensure_db().await?;
    let _res = sqlx::query!(
        "INSERT INTO history (method, url, output)
        VALUES (?, ?, ?)",
        req.method,
        req.url,
        res.output
    )
    .execute(&pool)
    .await?;

    Ok(())
}

#[async_std::main]
async fn main() {
    let req = try_parse_request();
    if let Err(ref err) = req {
        eprintln!("[curl-history] failed to parse request from args: {}", err);
    }

    let res = try_run_curl();
    let exit_code = res
        .as_ref()
        .map(|x| x.exit_code)
        .unwrap_or(STATUS_ERR_SPAWN);
    if let Err(ref err) = res {
        eprintln!("[curl-history] failed to run curl: {}", err);
    }

    if let (Ok(req), Ok(res)) = (req, res) {
        if let Err(err) = save_request(req, res).await {
            eprintln!("[curl-history] failed to save request: {}", err);
        }
    }

    std::process::exit(exit_code);
}
