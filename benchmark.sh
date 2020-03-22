#!/bin/bash
cargo build --release

TARGET="./target/release/rscompress-huffman"
COMMIT="$(git rev-parse --verify HEAD)"
OUTPUTFOLDER="benchmarks/${COMMIT::7}"

if [[ ! -e $OUTPUTFOLDER ]]; then
  mkdir -p $OUTPUTFOLDER
fi

INPUTS=(
  "test.tmp"
  #"../../Developments/big_files/2016020100-ART-chemtracer_grid_DOM02_ML_0018.nc"
)
OUTPUTS=(
  "/dev/null"
  "/tmp/ns"
)

HYPERFINE_RUNS=10
HYPERFINE_DROP_CACHES=false
HYPERFINE_EXPORT_FILE="${OUTPUTFOLDER}/hyperfine.${COMMIT::7}.json"
HYPERFINE_OUTPUTS=$(printf ",%s" "${OUTPUTS[@]}")
HYPERFINE_OUTPUTS="${HYPERFINE_OUTPUTS:1}"
HYPERFINE_CLEANUP=true
echo "Hyperfine output: ${HYPERFINE_EXPORT_FILE::(-5)}.<input>.<cacheinfo>.svg"


FLAMEGRAPH_EXPORT_FILE="${OUTPUTFOLDER}/flamegraph.${COMMIT::7}.svg"
FLAMEGRAPH_TARGET="${TARGET:17}"
FLAMEGRAPH_PERF_CLEANUP=true
echo "Flamegraph output: ${FLAMEGRAPH_EXPORT_FILE::(-4)}.<input>.<output>.svg"


VALGRIND_EXPORT_FILE="${OUTPUTFOLDER}/valgrind.${COMMIT::7}.massif"
echo "Valgrind output: ${VALGRIND_EXPORT_FILE::(-7)}.<input>.<output>.massif"


if [ `git rev-parse --abbrev-ref HEAD` == "development" ]; then
   echo "development-script"
elif [ `git rev-parse --abbrev-ref HEAD` == "master" ]; then
  for i in "${INPUTS[@]}"; do
    # Hyperfine
    echo "Running hyperfine for input: ${i}"
    if [ "${HYPERFINE_DROP_CACHES}" = true ]; then
      DROP_CACHE_CMD="sync; echo 3 | sudo tee /proc/sys/vm/drop_caches"
      OUTPUT="${HYPERFINE_EXPORT_FILE::(-5)}.${i}.cachedropped.json"
      hyperfine "${TARGET} ${i} {outs}" -s none -p "${DROP_CACHE_CMD}" -r $HYPERFINE_RUNS  --export-json $OUTPUT -L outs "${HYPERFINE_OUTPUTS}"
    else
      OUTPUT="${HYPERFINE_EXPORT_FILE::(-5)}.${i}.cachevalid.json"
      hyperfine "${TARGET} ${i} {outs}" -s none -r $HYPERFINE_RUNS  --export-json $OUTPUT -L outs "${HYPERFINE_OUTPUTS}"
    fi
    for o in "${OUTPUTS[@]}"; do
      echo "Running flamegraph for input: ${i} & output: ${o}"
      OUTPUT="${FLAMEGRAPH_EXPORT_FILE::(-4)}.${i}.${o//\//\-}.svg"
      cargo-flamegraph flamegraph --bin $FLAMEGRAPH_TARGET -o $OUTPUT -- "${i}" "${o}"
      echo "Running valgrind for input: ${i} & output: ${o}"
      OUTPUT="${VALGRIND_EXPORT_FILE::(-7)}.${i}.${o//\//\-}.massif"
      valgrind --tool=massif --time-unit=B --massif-out-file=$OUTPUT $TARGET $i $o
    done
  done
elif [ `git rev-parse --abbrev-ref HEAD` == "staging" ]; then
   echo "staging-script"
elif [ `git rev-parse --abbrev-ref HEAD` == "production" ]; then
   echo "production-script"
fi


if [ "${HYPERFINE_CLEANUP}" = true ]; then
  for o in "${OUTPUTS[@]}"; do
    rm $o
  done
fi

if [ "${FLAMEGRAPH_PERF_CLEANUP}" = true ]; then
  rm perf.data perf.data.old
fi
