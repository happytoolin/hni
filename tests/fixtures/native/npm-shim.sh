#!/bin/sh
basedir=$(dirname "$(echo "$0" | sed -e 's,\\,/,g')")

if [ -x "$basedir/node" ]; then
  exec "$basedir/node" --no-warnings "$basedir/../hello/cli.js" "$@"
else
  exec node --no-warnings "$basedir/../hello/cli.js" "$@"
fi
