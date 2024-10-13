FROM php:8.3-bullseye as base
WORKDIR /app

######################################################
# Step 1 | Install Dependencies
######################################################
RUN apt-get update && apt-get install -y curl git unzip procps php-pgsql php-mysql

#RUN install-php-extensions gd bcmath pdo_mysql zip intl opcache pcntl redis swoole exif zip bz2 @composer
RUN curl -sSL https://github.com/mlocati/docker-php-extension-installer/releases/latest/download/install-php-extensions -o - | sh -s \
    bcmath pdo_mysql pdo_pgsql zip intl exif zip bz2 mbstring fileinfo @composer

COPY composer.json composer.lock /app/
RUN composer install --no-dev --no-scripts --no-autoloader

CMD service postgresql start && tail -f /dev/null
