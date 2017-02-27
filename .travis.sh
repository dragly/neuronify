#!/bin/sh -e

if [ -z "$SNAPCRAFT_CONFIG" ]; then
    exit 0
fi

mkdir -p "$HOME/.config/snapcraft"
echo $SNAPCRAFT_CONFIG > "$HOME/.config/snapcraft/snapcraft.cfg"

if docker run -v $HOME:/root -v $(pwd):/cwd snapcore/snapcraft sh -c 'cd /cwd/.snapcraft; snapcraft'; then
#    if [ "${TRAVIS_BRANCH}" = "edge" ]; then
    docker run -v $HOME:/root -v $(pwd):/cwd snapcore/snapcraft sh -c "cd /cwd/.snapcraft; snapcraft push *.snap --release edge"
#    elif [ "${TRAVIS_BRANCH}" = "master" ]; then
#        docker run -v $HOME:/root -v $(pwd):/cwd snapcore/snapcraft sh -c "cd /cwd; snapcraft push *.snap --release stable"
#    fi
fi
