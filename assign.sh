#!/bin/bash
DUMPFILE=$1

rm -rf temp
mkdir temp
cd temp

split -n l/20 $DUMPFILE

# remove duplicates
find -name 'x*' | parallel -I% --max-args 1 cargo run assign % | sort | uniq > data.csv

# exclude authors not in database
psql -h localhost -d postgres -U postgres << EOF
\copy(select id from authors) to './.authors' csv;
EOF

comm -23 <(cat ./data.csv | awk -F '     ' '{ print $2 }' | sort | uniq) <(cat ./.authors | sort) > .bad_authors

cat ./data.csv | grep -vf .bad_authors > ./data_cleaned.csv

# upload
psql -h localhost -d postgres -U postgres << EOF
truncate work_authors;
\copy work_authors from './data_cleaned.csv' delimiter E'\t' escape '\' csv;
EOF

rm ./.authors
rm ./.bad_authors
rm ./data.csv
rm ./data_cleaned.csv
