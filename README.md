# Timely - Time Tracker

A simple time tracking service that runs on Wasmer Deploy

## Develop

The repo has a development CLI that follows the xtask pattern.

It is available via `cargo xtask`, and also `cargo x`.

* `cargo x develop`: Start a wcgi-runner local server in watch mode.
  Also watches for changes to the server and automatically rebuilds.

## Resources

* [Postgrest API](https://postgrest.org/en/stable/api.html)
  REST API documentation for Postgrest.
  Supabase, the DB provider, uses Postgrest internally.
