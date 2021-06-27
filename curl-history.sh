local actual_curl=$(command -pv curl)
local config_dir=${CURL_HISTORY_DIR:-"$HOME/.config/curl-history"}

curl() {
	file_name="$(date +%s) ${@//[^-a-zA-Z0-9.:_= ]/~}"
	log_file="$config_dir/${file_name:0:50}.log"

	mkdir -p "$config_dir"
	echo -E "curl $@" >"$log_file"
	echo "" >>"$log_file"

	"$actual_curl" "$@" | tee -a "$log_file"
}

curlh() {
	INITIAL_QUERY="$1"
	RG_PREFIX="rg --files-with-matches --no-heading --smart-case"
	CURL_FROM_FILENAME="sed -E 's/^([0-9]*)(.*)\.log\$/curl\2 @\1/'"
	FILENAME_FROM_CURL="sed -E 's/curl(.*) @([0-9]*)/\2\1.log/'"
	(
		cd "$config_dir"
		FZF_DEFAULT_COMMAND="$RG_PREFIX '$INITIAL_QUERY' | $CURL_FROM_FILENAME" \
			fzf --bind "change:reload:$RG_PREFIX {q} | $CURL_FROM_FILENAME || true" \
				--disabled \
				--query "$INITIAL_QUERY" \
				--layout=reverse \
				--preview 'cat "$(echo {} | '"$FILENAME_FROM_CURL"')"'
	)
}
