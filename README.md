# curl-history

**Experimental**

Wrapper for curl, which stores a history of all requests/responses

## Usage

Install: TODO

Setup:

```
alias curl=curl-history curl
alias curlh=curl-history history
```

Then use curl as usual

Then use `curlh example.com` to list requests made to example.com

## Develop

Requires a test database for compile time SQL query validation. Setup like this:

```
cargo install sqlx-cli
sqlx database setup
```
