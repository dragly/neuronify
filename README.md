# Neuronify 2.0

Neuronify is an educational neural network app.
It gives a low entry point to simulation-based neuroscience.
Without prior computational experience, Neuronify makes it possible to work on neural simulations.

The original version of Neuronify is described in
[this paper published in eNeuro](https://doi.org/10.1523/ENEURO.0022-17.2017).
This repository contains the source code for Neuronify 2,
which is based on the same ideas and concepts as Neuronify 1,
but with new features and the ability to embed simulations into websites and online books.

Neuronify 1 was originally written in C++, while Neuronify 2 is written in Rust.

Each release on GitHub has builds for Linux, Windows and macOS,
and a web build that can be put in any folder on a web server.
To run Neuronify from the source code instead, run `cargo run --release -p neuronify`.

To run Neuronify in the browser,
install [trunk](https://trunkrs.dev) and run `trunk serve --port <port>` in the `neuronify` folder.
Adding `?file=examples/leaky.nfy` to the URL shows that file
the way it would look embedded on a web page, with the editor hidden.
It can be any legacy `.nfy` file or `.neuronify` save file the page can fetch.

Neuronify is licensed under the GNU General Public License version 3, see [LICENSE](LICENSE).
