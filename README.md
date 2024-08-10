<h1 align="center">steal_stickers_bot</h1>

<h2>Preparing</h2>

1. Install [Docker](https://docs.docker.com/get-docker/).
2. Get bot token from [@BotFather](https://t.me/BotFather).
3. Clone our repository:
```
git clone https://github.com/Nnenty/steal_stickers_bot
```
4. cd into catalog:
```
cd steal_stickers_bot
```

<h2>Run bot</h2>

1. Build docker image:
```
docker build -t steal_stickers_bot .
```
2. Run docker container:
```
docker run -e BOT_TOKEN=<YOUR_BOT_TOKEN> steal_stickers_bot ss_bot_container
```

<h4>
<strong>The bot is ready to work.</strong>
</h4>

<h2>License</h2>

Licensed under:
- MIT License ([LICENSE])