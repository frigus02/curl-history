local actual_curl=$(command -pv curl)
local config_dir=${CURL_HISTORY_DIR:-"$HOME/.config/curl-history"}
local history_size=${CURL_HISTORY_SIZE:-1000}

_curl_history_cleanup() {
	rg --files --sortr path "$config_dir" | \
		tail -n +$(($history_size + 1)) | \
		tr '\n' '\0' | \
		xargs --null --no-run-if-empty rm
}

curl() {
	file_name="$(date +%s) ${@//[^-a-zA-Z0-9.:_= ]/~}"
	log_file="$config_dir/${file_name:0:50}.log"

	mkdir -p "$config_dir"
	echo -E "curl $@" >"$log_file"
	echo "" >>"$log_file"

	"$actual_curl" "$@" | tee -a "$log_file"

	_curl_history_cleanup
}

curlh() {
	if [ "$1" = "-o" ]; then
		SEARCH_OUTPUT=true
		shift
	else
		SEARCH_OUTPUT=false
	fi

	INITIAL_QUERY="$*"
	CURL_FROM_FILENAME="sed -E 's/^([0-9]*)(.*)\.log\$/curl\2 @\1/'"
	FILENAME_FROM_CURL="sed -E 's/curl(.*) @([0-9]*)/\2\1.log/'"
	FZF_PREVIEW='cat "$(echo {} | '"$FILENAME_FROM_CURL"')"'

	if [ "$SEARCH_OUTPUT" = true ]; then
		RG_PREFIX="rg --files-with-matches --smart-case"
		(
			cd "$config_dir"
			FZF_DEFAULT_COMMAND="$RG_PREFIX '$INITIAL_QUERY' | $CURL_FROM_FILENAME" \
				fzf --bind "change:reload:$RG_PREFIX {q} | $CURL_FROM_FILENAME || true" \
					--disabled \
					--query "$INITIAL_QUERY" \
					--layout=reverse \
					--preview "$FZF_PREVIEW"
		)
	else
		(
			cd "$config_dir"
			FZF_DEFAULT_COMMAND="rg --files | $CURL_FROM_FILENAME" \
				fzf --query "$INITIAL_QUERY" \
					--layout=reverse \
					--preview "$FZF_PREVIEW"
		)
	fi
}
