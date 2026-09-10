# Super Router

## What is this

Super Router is a routing library designed to work with any language (given that it can be run and has io capabilities) by spawning it as a process, feeding it the body as well as other info, and getting back a json object with data to return to the user.

It is currently very early in development and could possibly have major bugs.

## How to Run:

Run `cargo run -- build example` to build the example code. You will need rust installed, java installed, typescript installed, and gcc installed to do the build. If you don't want to build one of them, in the settings you can put `"ignore": true`

To then run the code, use `cargo run -- run example`. To run this you will need python and node js, as well as the output files from the above build. After it runs, navigate to localhost:3000 to see the page
