actual_curl=$(command -v curl)
config_dir="$HOME/.config/curl-history"

mkdir -p "$config_dir"

curlh() {
	file_name="$(date +%s) ${@//[^-a-zA-Z0-9.:_= ]/~}"
	log_file="$config_dir/${file_name:0:50}.log"
	$actual_curl "$@" | tee "$log_file"
}
