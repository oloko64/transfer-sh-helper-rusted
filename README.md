# Transfer.sh helper Rusted

The idea of the script is to store your transfer.sh links and simplify its usage, so you can remember them later and know when they will expire, but now written in Rust.

## Features

- Store your Transfer.sh links in a sqlite3 file locally.
- Easily compress files and folders before uploading them.
- Simplify the transfer.sh usage via CLI.
- Automatically calculates expired links.
- Option to easily delete the file from their servers.

https://user-images.githubusercontent.com/49915167/231621653-dabb2416-e422-469e-8862-7bd7f750c2be.mp4

## THIS SCRIPT IS NOT OFFICIAL.

I am not the author of the Transfer.sh service and don't have any affiliation with it.

I am just a user from their services, so I wrote this script to help me remember my links.

### Check out [Transfer.sh](https://github.com/dutchcoders/transfer.sh) on Github and give them a star.

# Installation

```bash
cargo install transferhelper
```

That's it, you are ready to go.

## Usage

This script has a few commands, you can use them in your terminal:

### Upload a file:

```bash
transferhelper upload <file>
```

### Compress a folder or file and upload it:

By default it will compress with default compression level, which is 6.

```bash
transferhelper upload <file> -c
```

You can also define a compression level from 0-9, 0 is the fastest and 9 is the best compression.

```bash
transferhelper upload <file_or_folder> -c -l 9
```

### View your stored links:

```bash
transferhelper list
```

### View your stored delete links:

```bash
transferhelper list -d
```

### Delete a link:

```bash
transferhelper delete
```

After running this command it will ask you for the link you want to delete and if you want to delete from the cloud as well.

### Delete the database:

```bash
transferhelper drop
```

After running this command it will ask for confirmation.

### View help:

```bash
transferhelper -h
```

---

## Build the package from Github

1. Install Rust on your machine, [Rustup](https://rustup.rs/).

2. Clone [this repository](https://github.com/OLoKo64/transfer-sh-helper-rusted).

3. Inside the cloned folder run:
```bash
cargo install --path .
```
## Thats it

Now you can use the program from your terminal.

```bash
transferhelper -h
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
