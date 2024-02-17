#!/usr/bin/bash
MARAIN_SRC_FOLDER=$1
if [[ -z $MARAIN_SRC_FOLDER ]]; then 
    echo 'No source folder provided.'
    exit 1
fi

echo "Attempting to find running Marain Service"
pgrep 'marain-server' || echo 'Marain Service not running.'

pgrep 'marain-server' && {
    echo 'stopping Marain Service.'
    pgrep 'marain-server' | xargs kill -9 2>/dev/null
}

{   
    echo "Moving to source directory: $MARAIN_SRC_FOLDER"
    cd $MARAIN_SRC_FOLDER || {
        echo "Could not find $MARAIN_SRC_FOLDER"
        exit 1
    }

    echo "Restarting Marain service."
    cargo run --release --locked
    export MARAIN_HOOK="1337"
    ./target/release/marain-server 2>>$MARAIN_SRC_FOLDER/marain.err.log 1>>$MARAIN_SRC_FOLDER/marain.log
}

