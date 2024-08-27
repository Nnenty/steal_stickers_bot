docker-build:
    docker build -t steal_stickers_bot .

auth: docker-build
    docker run -it --rm \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot steal_stickers_bot \
        auth

# If you want test your bot manually using stdin
run: docker-build
    docker run -it \
        --log-driver local --log-opt max-size=100m \
        --mount type=bind,source=./configs,target=/app/configs \
        --name steal_stickers_bot steal_stickers_bot \
        run
