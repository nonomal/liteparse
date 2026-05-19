for f in profiling/*.py
do
    stem="${f%.*}"
    scalene run $f --outfile "${stem}.json"
done
