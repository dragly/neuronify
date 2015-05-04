#!/bin/bash

MESSAGE=$@
echo $MESSAGE
if [ -z "$MESSAGE" ]
then
  echo "Error: You need to provide a commit message."
  echo "Usage:"
  echo "    ./publish_docs.sh <commit message>"
  exit 1
fi
DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ROOTDIR=$DIR/../..
DOCSDIR=$DIR/../../neuronify-docs

if [ -d $DOCSDIR/.git ]; then
  echo "All good, found neuronify-docs with git repo."
  cd $DOCSDIR
  git add *.html
  git add images
  git add scripts
  git add style
  git commit -am "$MESSAGE"
  git push
elif [ -d $DOCSDIR ]; then
  echo "Directory $DOCSDIR exists, but is not a git repo. Please delete the directory and run make_docs.sh first."
  exit 1
else
  echo "Could not find compiled docs. Please run make_docs.sh first."
fi
