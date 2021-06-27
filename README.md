# curl-history

**Experimental**

Wrapper for curl, which stores a history of all requests/responses

## Install

Download and source the shell script:

```sh
git clone --depth=1 https://github.com/frigus02/curl-history.git ~/.curl-history
source ~/.curl-history/curl-history.sh
```

It will export 2 functions:

- `curl`: a wrapper, which records the output of all executions into a file in `$CURL_HISTORY_DIR`
- `curlh`: let's you search your curl history

It requires the following external tools:

- [fzf](https://github.com/junegunn/fzf)
- [ripgrep](https://github.com/BurntSushi/ripgrep)

## Usage

Use `curl` as usual.

Then use `curlh` to search for requests and view responses. Examples:

- `curlh example.com`: search for requests where "example.com" is part of the command
- `curlh -o json`: search for requests where "json" is part of the command or output

## Configuration

You can set the following environment variables before sourcing the script.

- `$CURL_HISTORY_DIR` (defaults to `~/.config/curl-history`)
- `$CURL_HISTORY_SIZE` (defaults to `1000`)
