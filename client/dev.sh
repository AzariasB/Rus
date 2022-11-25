#!/bin/sh

out_dir="../api/static/scripts/"

js="${out_dir}main.js"

while inotifywait -e close_write ./src/**
do
  elm make --output=$js src/Main.elm
  echo "Initial size: $(cat $js | wc -c) bytes  ($js)"
done
