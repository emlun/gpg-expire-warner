# gpg-expire-warner

[![Crates.io](https://img.shields.io/crates/v/gpg-expire-warner.svg)](https://crates.io/crates/gpg-expire-warner)
[![Crates.io](https://img.shields.io/crates/d/gpg-expire-warner.svg)](https://crates.io/crates/gpg-expire-warner)
[![license](https://img.shields.io/crates/l/gpg-expire-warner.svg)](https://github.com/emlun/gpg-expire-warner/blob/master/LICENSE)
[![Build status](https://github.com/emlun/gpg-expire-warner/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/emlun/gpg-expire-warner/actions/workflows/build.yml?query=branch%3Amain)

Prints a warning when GPG keys are about to expire, and optionally helps update
their expiration time.


## Example

```
$ exec bash
The following GPG keys will expire soon:
2C7D8465F19C3CDC26237087BFD86BE9948C849A: 5 days
57E0FE20CF9F7BF57769909C0252D762936969DD: -20 days
```


## Usage

 1. Build and install using [`cargo`][cargo]:

    ```
    $ cargo install gpg-expire-warner
    ```

 2. Add something like this to your shell startup script (`.bashrc` or similar):

    ```
    gpg-expire-warner --days 14 \
                      "2C7D8465F19C3CDC26237087BFD86BE9948C849A" \
                      "0E70A5BEFD6E37F6EC272A025A5B6A61618EA60D" \
                      "57E0FE20CF9F7BF57769909C0252D762936969DD"
    ```

 3. When it's time to extend validity, you can add `--expire <expire>` to the
    command to automatically invoke `gpg` to update the expiration time of each
    key about to expire:

    ```
    $ gpg-expire-warner --days 14 \
                        "2C7D8465F19C3CDC26237087BFD86BE9948C849A" \
                        "0E70A5BEFD6E37F6EC272A025A5B6A61618EA60D" \
                        "57E0FE20CF9F7BF57769909C0252D762936969DD" \
                        --expire 1y

    The following GPG keys will expire soon:
    2C7D8465F19C3CDC26237087BFD86BE9948C849A: 5 days
    57E0FE20CF9F7BF57769909C0252D762936969DD: -20 days
    Extending validity by 1y for subkeys: 2C7D8465F19C3CDC26237087BFD86BE9948C849A, 57E0FE20CF9F7BF57769909C0252D762936969DD
    ```

    `<expire>` may be any expiration time format recognized by `gpg
    --quick-set-expire`.

 4. Move on and do better things than worry about your GPG keys expiring.


## License

GNU General Public License, version 3 or later.


[cargo]: https://crates.io/crates/gpg-expire-warner
