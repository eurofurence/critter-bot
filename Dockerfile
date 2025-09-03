FROM rust:1.89-bullseye as builder
WORKDIR /build
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/critter-bot /usr/local/bin/critter-bot
CMD ["critter-bot"]

# ######################################################
# # Step 1 | Install Dependencies
# ######################################################
# # RUN apt-get update && apt-get install -y curl git unzip procps php-pgsql php-mysql

# #RUN install-php-extensions gd bcmath pdo_mysql zip intl opcache pcntl redis swoole exif zip bz2 @composer
# # RUN curl -sSL https://github.com/mlocati/docker-php-extension-installer/releases/latest/download/install-php-extensions -o - | sh -s \
#     # bcmath pdo_mysql pdo_pgsql zip intl exif zip bz2 mbstring fileinfo @composer

# COPY composer.json composer.lock /app/
# RUN composer install --no-dev --no-scripts --no-autoloader

# CMD service postgresql start && tail -f /dev/null
