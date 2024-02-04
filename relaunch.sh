#!/usr/bin/bash
MARAIN_SRC_FOLDER=$1
if [[ -z $MARAIN_SRC_FOLDER ]]; then 
    echo 'No source folder provided.'
    exit 1
fi

echo "Attempting to find running Marain Service"
pgrep 'marain' || echo 'Marain Service not running.'

pgrep 'marain' && {
    echo 'stopping Marain Service.'
    pgrep 'marain' | xargs kill -9 2>/dev/null
}

{   
    echo "Moving to source directory: $MARAIN_SRC_FOLDER"
    cd $MARAIN_SRC_FOLDER || {
        echo "Could not find $MARAIN_SRC_FOLDER"
        exit 1
    }

    echo "Restarting Marain service."
    cargo run --release
}

