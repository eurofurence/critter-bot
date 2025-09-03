[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://choosealicense.com/licenses/mit/)
[![Create and publish Docker image](https://github.com/eurofurence/critter-bot/actions/workflows/docker.yml/badge.svg)](https://github.com/eurofurence/critter-bot/actions/workflows/docker.yml)


# Critter Bot

A Telegram bot for the critter system for receiving shift informations faster and for easier planning and managament.


## Contributing

Contributions are always welcome!

See `CONTRIBUTING.md` for ways to get started.

Please adhere to this project's `code of conduct`.


## Documentation

```
  Usage: critter-bot [OPTIONS] --pool <pool> --token <token> --critter-token <critter-token> --critter-baseurl <critter-baseurl> [pollint] [pq-lim]

Arguments:
  [pollint]  Interval between every poll to the critter server to sync up tasks in seconds [env: POLLINT=] [default: 60]
  [pq-lim]   Sets a limit to how many user lookups the tool can make at once in the database [env: PARALLEL_LOOKUP_LIMIT=] [default: 16]

Options:
  -p, --pool <pool>
          Postgres database url [env: DATABASE_URL=]
  -t, --token <token>
          Token to be use for telegram bot [env: TELEGRAM_TOKEN=]
  -c, --critter-token <critter-token>
          Token to be use for talking to the crittersystem [env: CRITTER_TOKEN=]
      --critter-baseurl <critter-baseurl>
          Baseurl of crittersystem: e.g. `https://critter.eurofurence.org/` [env: CRITTER_BASEURL=] [default: https://critter.eurofurence.org/]
      --no-migrate
          Prevents migrations from running on bot start, potentially unsafe! [env: NO_MIGRATE=]
  -z, --timezone <timezone>
          Sets the events timezone using a TZ identifier code, such as `Europe/Berlin` [env: TIMEZONE=] [default: Europe/Berlin]
  -h, --help
          Print help
```

Mandetory enviroment variables:
- `DATABASE_URL`
- `TELEGRAM_TOKEN`
- `CRITTER_TOKEN` (critter system api key)
- `CRITTER_BASEURL` (reference above for example)

## License

[MIT](https://choosealicense.com/licenses/mit/)

