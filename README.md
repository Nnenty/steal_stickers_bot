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
6. Copy [config.toml.example](./configs/config.toml.example), remove `.example` from name of file and fill it required information.
7. Copy [.env.example](./.env.example), remove `.example` from name of file and fill it ***the same*** required information as in your file `config.toml`.

<h2>Run bot</h2>

1. <h4>Authorize client</h4>

To authorize client run command below:
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
2. <h4>Run bot</h4>

To run bot use command below:
```
just compose-run
```
> [Download justfile](https://github.com/casey/just?tab=readme-ov-file#pre-built-binaries)

*or if you want run it manually:*
```
docker compose up
```

3. <h4>Migrate database</h4>

After running bot will start working, but database will be without migrations. To solve it, run command below with your database information (that you put into `.env` and `config.toml` files):
```
sqlx migrate run --source ./src/infrastructure/database/migrations --database-url="postgres://{username}:{password}@{host}:{port}/{db}"
```

<strong>If you encounter errors that are directly related to my code (docker errors, bot errors, etc.), please [open an Issue](https://github.com/Nnenty/steal_stickers_bot/issues/new). Thanks :)</strong>


<h2>License</h2>

Licensed under:
- MIT License ([LICENSE](./LICENSE) or https://opensource.org/license/MIT)