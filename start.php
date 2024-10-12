<?php
require_once './vendor/autoload.php';
require_once './app/main.php';
require_once './app/Database/database.php';

use Dotenv\Dotenv;
use App\Main;

$dotenv = Dotenv::createImmutable(__DIR__);
$dotenv->load();

new Main();
