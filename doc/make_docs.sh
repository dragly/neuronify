#!/bin/bash

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ROOTDIR=$DIR/../..
DOCSDIR=$DIR/../../neuronify-docs

if [ -d $DOCSDIR/.git ]; then
  echo "All good, found neuronify-docs with git repo."
  echo "Pulling changes in neuronify-docs (note: not pulling in the current folder)."
  cd $DOCSDIR
  if ! git reset --hard; then
      echo "Could not reset"
      exit 1
  fi
  if ! git pull; then
      echo "Could not pull"
      exit 1
  fi
elif [ -d $DOCSDIR ]; then
  echo "Directory $DOCSDIR exists, but is not a git repo. Please delete the directory first."
  exit 1
else
  cd $ROOTDIR
  if ! git clone -b gh-pages git@github.com:CINPLA/neuronify.git neuronify-docs; then
      echo "Could not clone"
      exit 1
  fi
fi

cd $DIR
echo Running $DOCSDIR/qdoc neuronify.qdocconf
LD_LIBRARY_PATH=$DOCSDIR $DOCSDIR/qdoc neuronify.qdocconf

