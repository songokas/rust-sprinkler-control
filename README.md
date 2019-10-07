# Control sprinkler system using mqtt messages

* arduino/esp clients sends messages with their pin statuses/readings
* system reacts according to configuration by sending messages to control nodes

## Dependencies

* mqtt clients providing messages
* mqtt clients reacting to messages

## How it works

* based on config.yml system sends mqtt messages to controller e.g "sprinkler/nodes/the-one/set/json" {"pin": 3, "set": 1}
* controller reacts by turning those pins on/off
* controller sends messages to topic with pin/sensor values "sprikler/nodes/the-one/current/digital/3" "1" for fine grained control

## Howto run

```
cargo test
cargo build --release

# provide your own configuration and test. example src/config.yml

./target/release/sprinkler-control --config src/config.yml

```

## Make it pernament

### systemctl

```
# become root

sudo bash

# change according to your needs

USER="tomas" CONFIG_PATH="`pwd`/src/config.yml" BIN_PATH="`pwd`/target/release/sprinkler-control" envsubst < "services/sprinkler-control.service" > /etc/systemd/system/sprinkler-control.service

systemctl daemon-reload

systemctl enable sprinkler-control
```
