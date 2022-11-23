#!/bin/sh

set -e

out_dir="../api/static/scripts/"

js="${out_dir}main.js"

while inotifywait -e close_write $@
do
  elm make --optimize --output=$js $@
  echo "Initial size: $(cat $js | wc -c) bytes  ($js)"
done
