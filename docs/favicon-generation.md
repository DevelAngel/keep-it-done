# Favicon Generation

## Preparation (Debian/Ubuntu)

```console
sudo apt-get update
sudo apt-get install icoutils
```

## ICO Generation

```console
$ icotool -c -o favicon.ico favicon-128x128.png favicon-64x64.png favicon-32x32.png favicon-16x16.png
```

Note: The browser seems to load the first picture in `ico` file.

Verify ICO file with the following command:

```console
$ icotool -l favicon.ico
```
