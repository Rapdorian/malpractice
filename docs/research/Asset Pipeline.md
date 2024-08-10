# Asset Pipeline Architecture

I need to find a good way of handling building assets.
ultimately we need to be able to store raw/editing file types
such that devs don't need to have access to them but have a
convenient way to access the final output files.

Essentiall I want a seperate art repo (may be a subdir in a single repo that
uses sparse checkout) I want that art repo to have a build system that creates
final artifacts and uploads them somewhere then we want the build system for
the code side to be able to fetch those build artifacts 

These build time artifacts could be fetched in a build.rs script.
Although I am planning on using cargo-make to handle builds so I think just
a cli program is enough.
The big problem is that I don't want to spend too much on cloud storage/server
hosting and I do like the idea of the whole project being able to be built
locally.

Also since I'm a single dev I don't think I'm going to be saving much of
anything by being able to have a sparse-checkout of the art repo. I'll have to
have it on my machine regardless.

I think pretty much all of these problems stem from using cloud hosting. If I
self-hosted the repo I wouldn't have to care about size of things until it
becomes a problem. However that would make working from not home more
challenging. (Setup a VPN you idiot)
