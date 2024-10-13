<?php

use Dotenv\Dotenv;

$dotenv = Dotenv::createImmutable(__DIR__);
$dotenv->load();

switch ($_ENV['DB_CONNECTOR']) {
    case 'pgsql':
        return [
            'dbname' => $_ENV['DB_DATABASE'],
            'user' => $_ENV['DB_USERNAME'],
            'password' => $_ENV['DB_PASSWORD'],
            'host' => $_ENV['DB_HOST'],
            'driver' => 'pdo_pgsql',
            'port' => $_ENV['DB_PORT'],
        ];
        break;

    case 'mysql':
        return [
            'dbname' => $_ENV['DB_DATABASE'],
            'user' => $_ENV['DB_USERNAME'],
            'password' => $_ENV['DB_PASSWORD'],
            'host' => $_ENV['DB_HOST'],
            'driver' => 'pdo_mysql',
            'port' => $_ENV['DB_PORT'],
        ];
        break;
    default:
        throw new Exception("Unknown database connector");
        break;
}
