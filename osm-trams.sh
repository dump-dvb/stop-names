#! /usr/bin/env bash

curl http://overpass-api.de/api/interpreter -X POST -d@- <<EOF
[out:json][timeout:25];
(
  rel(399665);
  >>;
);
out body;
out skel qt;
EOF
