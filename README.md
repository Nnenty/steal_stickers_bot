<h1 align="center">steal_stickers_bot</h1>
<div align="center">
<h4><a href="https://t.me/steal_stickers_bot">bot in Telegram</a>
</div>

<h2>Preparing</h2>

1. Install [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/).
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
6. Fill [config.toml.example](./configs/config.toml.example) file with your information and remove `.example` from name.

<h2>Run bot</h2>


1. Authorize client: 
```
just auth
```
> [Download justfile](https://github.com/casey/just?tab=readme-ov-file#pre-built-binaries)

*or if you want run it manually:*
```
docker build -t steal_stickers_bot . && \
docker run -it --rm \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot steal_stickers_bot \
        auth
```
> After you have launched Docker, a code should be sent to your Telegram account.
Enter this code into your terminal.
2. Run bot:
```
docker compose up --build
```
*or if you want run it manually:*
```
# --rm at your discretion
docker run --rm \
        --log-driver local --log-opt max-size=100m \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot steal_stickers_bot \
        run
```

<strong>If you encounter errors that are directly related to my code (docker errors, bot errors, etc.), please [open an Issue](https://github.com/Nnenty/steal_stickers_bot/issues/new). Thanks :)</strong>


<h2>License</h2>

Licensed under:
- MIT License ([LICENSE](./LICENSE) or https://opensource.org/license/MIT)