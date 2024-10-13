<?php
namespace App;

use Psr\Log\NullLogger;
use SergiX44\Nutgram\Nutgram;
use App\Commands\StartCommand;
use App\Database\Database as DB;
use SergiX44\Nutgram\Configuration;
use SergiX44\Nutgram\RunningMode\Webhook;
use SergiX44\Nutgram\Logger\ConsoleLogger;

class Main {

    private $commands = [
        StartCommand::class
    ];

    public function __construct() {
        $this->main();
    }

    private function main() {
        echo "Initializing...\r\n";

        echo "Loading config...\r\n";
        $config = new Configuration(
            botName: $_ENV['BOT_NAME'],
            logger: ($_ENV['DEBUG'] === 'true') ? ConsoleLogger::class : NullLogger::class //Logs Telegram requests, everything else needs to be logged with an custom logger
        );

        $bot = new Nutgram($_ENV['TELEGRAM_TOKEN'], $config);

        echo "Setting running mode...\r\n";
        //$bot->setRunningMode(Webhook::class);

        echo "Registering commands...\r\n";
        //$bot->registerCommand(StartCommand::class);

        echo "Testing database connection...\r\n";
        if (!DB::test()) {
            echo "[ERROR] No connection to the database\r\n";
            return;
        }

        #Testing
        $bot->onCommand('start', function (Nutgram $bot) {
            $bot->sendMessage('Your chat id is ' . $bot->chatId());
        });
        $bot->registerMyCommands();

        echo "Starting bot...\r\n";
        $bot->run();
        echo "Bot started\r\n";
    }
}
