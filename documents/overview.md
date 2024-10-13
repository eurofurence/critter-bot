# Overview

## Table of contents

[TOC]

## General information

WIP

## Installation

Insert composer info here ^^ (Reminder)

## Commands

> [!NOTE]
>
> Commands has to be run in the same folders where `start.php` is located

#### Bot

###### Starting the bot

```
php start.php
```



#### Database

###### List available migration commands

```
./vendor/bin/doctrine-migrations
```



## Database

*WIP*



## Docker

The Docker container provides an ***pgSQL*** database, the credentials are in the `.env.example` file

###### Build and run docker container

```
docker-compose up --build
```

> [!NOTE]
>
> Command has to be run in the same folders where `start.php` is located



## Dependencies

- Migrations: [doctrine/migrations: Doctrine Database Migrations Library (github.com)](https://github.com/doctrine/migrations) (License: MIT)
- Telegram Bot Framework: [nutgram/nutgram: The Telegram bot framework that doesn't drive you nuts. (github.com)](https://github.com/nutgram/nutgram) (License: MIT)
- .ENV Library: [vlucas/phpdotenv: Loads environment variables from `.env` to `getenv()`, `$_ENV` and `$_SERVER` automagically. (github.com)](https://github.com/vlucas/phpdotenv) (License: BSD 3-Clause License)