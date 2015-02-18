# Neuronify

Visual neuron simulator using integrate-and-fire neurons.

## Building Neuronify

### Prerequisites

1. Download Qt from http://www.qt.io/download-open-source/
2. Run the installer and follow the instructions

### Downloading and building

3. Clone the git repo: git@github.com:CINPLA/neuronify.git
4. Open Qt
5. In Qt, use Open Project, go to the neuronify folder and open nestify.pro or neuronify.pro, whichever is present
6. Under configure project that shows up, mark Desktop Qt 5.4.0 GCC and unmark Desktop before pressing Configure Project
7. Press Run (green triangle) and it should hopefully build and run.

### Creating a deployable .dmg on Mac

Simply run the following command in the build-directory:

  /path/to/Qt/installation/clang_64/bin/macdeployqt neuronify.app -qmldir=../neuronify -dmg
  
This should create a .dmg that can be used on machines without Qt installed.
