#!/bin/bash

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ROOTDIR=$DIR/../..
DOCSDIR=$DIR/../../neuronify-docs

if [ -d $DOCSDIR/.git ]; then
  echo "All good, found neuronify-docs with git repo."
  echo "Pulling changes in neuronify-docs (note: not pulling in the current folder)."
  cd $DOCSDIR
  git pull
elif [ -d $DOCSDIR ]; then
  echo "Directory $DOCSDIR exists, but is not a git repo. Please delete the directory first."
  exit 1
else
  cd $ROOTDIR
  git clone -b gh-pages git@github.com:CINPLA/neuronify.git neuronify-docs
fi

cd $DIR
qdoc neuronify.qdocconf
