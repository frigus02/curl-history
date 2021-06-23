use crate::{db::ensure_db, BoxError};
use std::ffi::OsString;
use structopt::{clap::AppSettings, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(settings = &[
    AppSettings::NoBinaryName,
    AppSettings::DisableVersion,
])]
struct Opts {
    #[structopt(long)]
    id: Option<i64>,
    #[structopt(required = true, conflicts_with = "id")]
    terms: Vec<String>,
}

#[derive(Debug)]
struct Record {
    id: i64,
    method: String,
    url: String,
    output: String,
}

async fn search_by_id(id: i64) -> Result<Vec<Record>, BoxError> {
    let pool = ensure_db().await?;
    let record = sqlx::query_as!(Record, "SELECT * FROM history WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;
    Ok(record.map(|x| vec![x]).unwrap_or_else(Vec::new))
}

async fn search_by_terms(terms: Vec<String>) -> Result<Vec<Record>, BoxError> {
    let pool = ensure_db().await?;
    let records = sqlx::query_as!(
        Record,
        "SELECT * FROM history WHERE method LIKE ? OR url LIKE ?",
        terms[0],
        terms[0]
    )
    .fetch_all(&pool)
    .await?;
    Ok(records)
}

pub async fn search(args: Vec<OsString>) {
    let opts = Opts::from_iter(args);
    let result = if let Some(id) = opts.id {
        search_by_id(id).await
    } else {
        search_by_terms(opts.terms).await
    };
    match result {
        Ok(records) => println!("{:#?}", records),
        Err(err) => eprintln!("[curl-history] error searching history: {}", err),
    };
}
