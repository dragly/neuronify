#!/bin/bash
for x in 36 40 48 72 76 80 96 120 144 152 180 192 512 1024 ; do inkscape --export-png android/neuronify_logo_${x}.png -w ${x} neuronify_logo.svg ; done
