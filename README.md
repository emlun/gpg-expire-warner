# gpg-expire-warner

Prints a warning when GPG keys are about to expire


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

 3. Move on and do better things than worry about your GPG keys expiring.


## License

GNU General Public License, version 3 or later.


[cargo]: https://crates.io/crates/gpg-expire-warner
