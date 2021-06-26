actual_curl=$(command -pv curl)
config_dir="$HOME/.config/curl-history"

mkdir -p "$config_dir"

curl() {
	file_name="$(date +%s) ${@//[^-a-zA-Z0-9.:_= ]/~}"
	log_file="$config_dir/${file_name:0:50}.log"
	echo -E "curl $@" >"$log_file"
	echo "" >>"$log_file"
	$actual_curl "$@" | tee -a "$log_file"
}

curlh() {
	INITIAL_QUERY="$1"
	RG_PREFIX="rg --files-with-matches --no-heading --smart-case "
	(
		cd "$config_dir"
		FZF_DEFAULT_COMMAND="$RG_PREFIX '$INITIAL_QUERY'" \
			fzf --bind "change:reload:$RG_PREFIX {q} || true" \
				--disabled \
				--query "$INITIAL_QUERY" \
				--layout=reverse \
				--preview 'cat {}'
	)
}
