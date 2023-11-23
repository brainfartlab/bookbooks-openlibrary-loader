#!/bin/bash
DUMPFILE=$1
TABLE=$2

rm -rf temp
mkdir temp
cd temp

split -n l/20 $DUMPFILE
#ls | parallel -I% --max-args 1 cargo run % | sed 's/\\"/\\\\"/g' | sed 's/""/\\"/g' > data.csv
find -name 'x*' | parallel -I% --max-args 1 cargo run format % > data.csv
psql -h localhost -d postgres -U postgres << EOF
truncate $TABLE;
\copy $TABLE from './data.csv' delimiter E'\t' escape '\' csv;
EOF
