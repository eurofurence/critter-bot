<?php
require_once './vendor/autoload.php';
require_once './app/main.php';

use SergiX44\Nutgram\Nutgram;
use Dotenv\Dotenv;
use main\Main;

$dotenv = Dotenv::createImmutable(__DIR__);
$dotenv->load();

new Main();
