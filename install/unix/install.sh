# INSTALL
echo "[RUNNING::INSTALL]"

export SERF_ROOT_DIR="$HOME/.serf"
TARGET_DIR="$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")/../../target/release"

while getopts td:rd: flag
do
    case "${flag}" in
        rd) export SERF_ROOT_DIR=${OPTARG};;
    esac
done

echo "[RUNNING::BUILD] ROOT=$SERF_ROOT_DIR"
echo "[BUILD_TARGET] ARTIFACTS=$TARGET_DIR"

cargo build --release

echo "[COMPLETE::BUILD]"
echo "[RUNNING::COPY_EXECUTABLES] ROOT=$SERF_ROOT_DIR"

# check dir existence and make if not exist
if [ ! -d "$SERF_ROOT_DIR" ]; then
    mkdir $SERF_ROOT_DIR
fi

cp -t $SERF_ROOT_DIR $TARGET_DIR/sqlite_server_srv $TARGET_DIR/sqlite_server_cli

echo "[COMPLETE::COPY_EXECUTABLES]"

cargo clean -vv --release
echo "[COMPLETE::INSTALL]"

exit 0

