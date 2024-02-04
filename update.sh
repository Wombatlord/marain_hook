#!/usr/bin/bash
MARAIN_SRC_FOLDER=$1
if [[ -z $MARAIN_SRC_FOLDER ]]; then 
    echo 'No source folder provided.'
    exit 1
fi

{   
    echo "Moving to source directory: $MARAIN_SRC_FOLDER"
    cd $MARAIN_SRC_FOLDER || {
        echo "Could not find $MARAIN_SRC_FOLDER"
        exit 1
    }

    echo "Pulling copy of latest source code."
    git pull origin main || {
        echo "Could not pull updated source code."
        exit 1
    }
}

