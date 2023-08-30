# 🦀 Bitcoin Miner ⛏️

> Майнер биткоинов без ху🦆ни.

*Пока что поддерживаем 21 версию [Bitcoin Core](https://bitcoin.org/en/bitcoin-core/)*

[Как это работает?](/HOWITWORKS.md)

## 🚀 Быстрый старт


```ini
# bitcoin.conf
listen=1
# blocksonly=1

rpcuser=user
rpcpassword=pass
```

```ini
# .env
chain=main
rpcuser=user
rpcpassword=pass
script_pubkey=<your_script_pubkey>
```

```bash
cargo run --release
```

## 🧐 Полный гайд

> Примеры сделаны для MacOS 🍏
>
> Для Windows 🪟 и Linux 🐧 меняется процесс установки и расположение Bitcoin Core файлов.

### 1️⃣ Установка

1. [Bitcoin Core](https://bitcoin.org/en/version-history) (обязательно с bitcoind и bitcoin-cli) + [гайд по установке](https://bitcoin.org/en/full-node#mac-os-x-instructions)

2. Rust 🦀 и текущий репозиторий с майнером

### 2️⃣ Определяем сеть ⛓️ для запуска

<kbd>mainnet</kbd> - **оригинальная и основная сеть** для транзакций биткоинов.

<kbd>testnet</kbd> - **глобальная среда тестирования**, в которой биткоины не имеют реальной ценности.
[Их даже меняли на деньги 🫠](https://en.bitcoin.it/wiki/Testnet#:~:text=Testnet2%20was%20just%20the%20first%20testnet%20reset%20with%20a%20different%20genesis%20block%2C%20because%20people%20were%20starting%20to%20trade%20testnet%20coins%20for%20real%20money.)

<kbd>regtest</kbd> (режим регрессионного тестирования) - **локальная среда тестирования**, в которой разработчики могут генерировать блоки по запросу.

#### Различия на старте:

- Если <kbd>mainnet</kbd> или <kbd>testnet</kbd> надо **ждать синхронизации**:

  - <kbd>mainnet</kbd> - примерно 2 дня;

  - <kbd>testnet</kbd> - до 1 дня.

- Если <kbd>regtest</kbd> для работы надо будет **поднять две ноды** (инструкция ниже).

### 3️⃣ Настраиваем сеть ⛓️

#### Для <kbd>regtest</kbd> ⚠️

> Проще всего проверить майнер 👍

Создаем два файла *bitcoin.conf* для основной ноды и вспомогательно (для подтверждения блоков)

```ini
# $HOME/Library/Application Support/Bitcoin/bitcoin.conf
regtest=1
listen=1

[regtest]
rpcuser=user
rpcpassword=pass
fallbackfee=0.0001
addnode=localhost:10000
```

```ini
# $HOME/Library/Application Support/Bitcoin/regtest2/bitcoin.conf
regtest=1
listen=1

[regtest]
rpcuser=user
rpcpassword=pass
fallbackfee=0.0001
port=10000
rpcport=20000
```

Поднимаем сначала вторую ноду, чтобы основная смогла сразу её подключить

```bash
bitcoind -daemon --datadir="$HOME/Library/Application Support/Bitcoin/regtest2"
```

#### Для <kbd>testnet</kbd> ⚠️

```ini
# $HOME/Library/Application Support/Bitcoin/bitcoin.conf
testnet=1
listen=1

[testnet]
rpcuser=user
rpcpassword=pass
```

#### Для <kbd>mainnet</kbd> ⚠️

```ini
# $HOME/Library/Application Support/Bitcoin/bitcoin.conf
listen=1

rpcuser=user
rpcpassword=pass
```

### 4️⃣ Запускаем ноду

```bash
bitcoind -daemon
```
### 5️⃣ Создаем кошелек и адрес

Используем *bitcoin-cli* для вызова rpc комманд -
[createwallet](https://developer.bitcoin.org/reference/rpc/createwallet.html)
и
[getnewaddress](https://developer.bitcoin.org/reference/rpc/getnewaddress.html)

```bash
bitcoin-cli createwallet <name>
bitcoin-cli getnewaddress <label>
```

### 6️⃣ Создаем *.env* файл

**url** - стандартное значение для *bitcoind*

**rpcuser** и **rpcpassword** - указываем из *bitcoin.conf*

**script_pubkey** - получим при вызовае [getaddressinfo](https://developer.bitcoin.org/reference/rpc/getaddressinfo.html) для нужного адреса

```bash
bitcoin-cli getaddressinfo <address>
```

```ini
# .env
url=localhost:18443
rpcuser=user
rpcpassword=pass
script_pubkey=<your_script_pubkey>
```

### 7️⃣ Поднимаем сеть ⛓️  только для <kbd>regtest</kbd> ⚠️

Генерим первые блоки для запуска сети

```bash
bitcoin-cli generatetoaddress 101 $(bitcoin-cli getnewaddress)
bitcoin-cli getbalance

bitcoin-cli sendtoaddress $(bitcoin-cli getnewaddress) 10.00
bitcoin-cli listunspent 0
bitcoin-cli -generate 1
```

### 8️⃣ Проверяем запрос для майнинга

[getblocktemplate](https://developer.bitcoin.org/reference/rpc/getblocktemplate.html)

```bash
bitcoin-cli getblocktemplate '{"rules": ["segwit"]}'
```

### 9️⃣ Запускаем майнер

```bash
cargo run --release
```

### 🔟 Отрубон

```bash
bitcoin-cli stop
# так же для второй ноды если был regtest
bitcoin-cli -rpcport=20000 stop
```
