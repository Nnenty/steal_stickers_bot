docker-build:
    docker build -t steal_stickers_bot .

auth:
    docker run -it --rm \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot nnenty/steal_stickers_bot:latest \
        auth

run:
    docker run --rm \
        --log-driver local --log-opt max-size=100m \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot nnenty/steal_stickers_bot:latest \
        run

compose-run:
    docker compose up

compose-run-build:
    docker compose up --build
