#! /usr/bin/env bash

curl http://overpass-api.de/api/interpreter -X POST -d@- <<EOF
[out:json][timeout:25];
(
  rel(57543);
  >>;
);
out body;
out skel qt;
EOF
