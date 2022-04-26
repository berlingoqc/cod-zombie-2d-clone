
# cod-zombie-2d-clone


Trying to make a cod-zombie 2d clone playable in multiplayer in the browser , mobile and desktop.

[Demo site](https://berlingoqc.github.io/cod-zombie-2d-clone/)

## Development logs

Here is a list of my development entry , when i code or work on the project
i tend to right summary to help me focusing.

* [First day, just thinking](./logs/2022-04-13.md)
* [Starting the project](./logs/2022-04-14.md)
* [Trying to integrate a tiled library](./logs/2022-04-15.md)
* [Implementing the base of the game](./logs/2022-04-18.md)
* [Starting to look at the networking](./logs/2022-04-19.md)
* [Regretting my life resplitting the code](./logs/2022-04-20.md)
* [Setuping the base of the server project](./logs/2022-04-23.md)
* [Implementing weapons](./logs/2022-04-25.md)
* [More weapon and starting windows](./logs/2022-04-26.md)

## Compile and run the game locally

### Client

```bash
# Move to the client repository to build and run it
cd client/

# Fast compile of the application with hot reload
# Compile the application and run it directly
cargo make run-dyn

# Build a modestly optimized build
cargo make build-native
./target/debug/cod-zombie-2d-clone

# Run in desktop mode in fast recompile
cargo make run-dyn

# Run in the browser
cargo make serve

# Alternative with live-server
npm install -g live-server

# In one terminal
cd public/
live-server
# In the other one you can restart the build
cargo make build-web
```

### Server

```bash
cd ./server/
cargo make run
```


## Work on the game without compiling

Download the latest version of the game in the releases section
of the github repository.

Download the repository and put the game in the root of the repository.

Start it and enjoy live reload.

