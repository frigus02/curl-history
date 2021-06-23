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
    let prepared_terms: Vec<_> = terms
        .into_iter()
        .map(|term| {
            let escaped = term
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            format!("%{}%", escaped)
        })
        .collect();
    let mut records = Vec::new();
    for prepared_term in prepared_terms {
        records.extend(
            sqlx::query_as!(
                Record,
                "SELECT history.* FROM history
                WHERE method LIKE $1 ESCAPE '\\'
                OR url LIKE $1 ESCAPE '\\'",
                prepared_term,
            )
            .fetch_all(&pool)
            .await?,
        );
    }

    records.sort_unstable_by_key(|record| std::cmp::Reverse(record.id));
    records.dedup_by_key(|record| record.id);

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
