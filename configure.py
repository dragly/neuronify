from sys import argv
import subprocess
import os

if len(argv) < 2:
    print("Usage: python configure.py <qmake path>")
    print("qmake path is typically /home/username/apps/Qt/5.6/gcc_64/bin/qmake")
    exit(0)

qmake_path = argv[1]

subprocess.call("git submodule init", shell=True)
subprocess.call("git submodule update", shell=True)

os.mkdir("libs/build-qtcharts")
os.chdir("libs/build-qtcharts")

subprocess.call(qmake_path + " ../qtcharts", shell=True)
subprocess.call("make -j4", shell=True)
subprocess.call("make install", shell=True)
