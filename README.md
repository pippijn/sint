# Sinterklaas game

Setup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

cd sint-ftl
scripts/start_game.py --verbose
```

This should install all the dependencies you need. If something goes wrong with
the Python venv, try `. .venv/bin/activate` and then running
`scripts/start_game.py` again. The `--verbose` flag makes `pip` print its
progress, which is useful the first time so you can see it's not doing nothing
(the first install can take 5-10 minutes).
