<h1 align="center">steal_stickers_bot</h1>
<div align="center">
<h4><a href="https://t.me/steal_stickers_bot">bot in Telegram</a>
</div>

<h2>Preparing</h2>

1. Install [Docker](https://docs.docker.com/get-docker/).
2. Create your Telegram application [following instructions](https://core.telegram.org/api/obtaining_api_id).
3. Get the bot token from [@BotFather](https://t.me/BotFather).
4. Clone our repository:
```
git clone https://github.com/Nnenty/steal_stickers_bot
```
5. Cd into catalog:
```
cd steal_stickers_bot
```
6. Fill [config.toml.example](./config.toml.example) file with your information and remove `.example` from name.

<h2>Run bot</h2>


1. Build docker image:
```
docker build -t steal_stickers_bot .
```
2. Run docker container:
> specify <b>your</b> bot token in env `BOT_TOKEN`!
```
docker run -it --name steal_stickers_bot_container steal_stickers_bot
```
3. After you have launched Docker, a code should be sent to your Telegram account. Enter this code into your terminal and if you did everything correctly, <strong>the bot will start working.</strong>.

If you encounter errors that are directly related to my code (docker errors, bot errors, etc.), please [open an Issue](https://github.com/Nnenty/steal_stickers_bot/issues/new). Thanks :)


<h2>License</h2>

Licensed under:
- MIT License ([LICENSE](./LICENSE))