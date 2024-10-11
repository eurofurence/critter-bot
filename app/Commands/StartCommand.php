<?php
namespace App\Commands;

use SergiX44\Nutgram\Handlers\Type\Command;
use SergiX44\Nutgram\Nutgram;

class StartCommand extends Command
{
    // Called on command "/start"
    protected string $command = 'start';

    // It's possible to set a description for the current command
    // this WILL be automatically registered
    protected ?string $description = 'A lovely start command';

    public function handle(Nutgram $bot): void
    {
        $bot->sendMessage('Hello there!');
    }
}
