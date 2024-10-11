<?php
namespace App;

use SergiX44\Nutgram\Nutgram;
use SergiX44\Nutgram\Configuration;
use SergiX44\Nutgram\RunningMode\Webhook;
use SergiX44\Nutgram\Logger\ConsoleLogger;
use App\Commands\StartCommand;

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
            testEnv: $_ENV['TEST_ENV'],
            logger: ConsoleLogger::class
        );

        $bot = new Nutgram($_ENV['TELEGRAM_TOKEN']);

        echo "Settings running mode...\r\n";
        //$bot->setRunningMode(Webhook::class);

        echo "Registering commands...\r\n";

        //$bot->registerCommand(StartCommand::class);


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
