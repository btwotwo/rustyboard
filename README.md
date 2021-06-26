# Rustyboard

## What is it?

This is a Rust reimplementation of the [nanoboard](https://github.com/nanoboard/nanoboard) project, which was abandoned a long time ago.

## What is a nanoboard?

Nanoboard is an [imageboard](https://en.wikipedia.org/wiki/Imageboard) with a twist. Unlike traditional imageboards, which are hosted somewhere on a server, nanoboard is hosted on other imageboards.

One of the fundamental principles of a nanoboard is the PNG image steganography. User posts are hidden inside so-called PNG containers which then are posted onto other imageboards. The lack of P2P connection allows users to spread nanoboard asynchronously, by posting containers with nanoposts to their favourite imageboards and keeping nanoboard alive forever.

If you are not convinced yet, take a look at nanoboard advantages!

### Nanoboard advantages
* Self moderation. You can delete every post from your copy of nanoboard, which will remove its contents from the disk, and exclude said post from containers.

* Potentially unkillable. There is no obvious way to detect nanopost collection is hidden inside a PNG image, so users can communicate via every image-posting service, as long as it doesn't compress the image. One of the most extreme ideas of the nanopost spreading, is to encode them into QR codes, print them, and put these codes everywhere!

* Familiar interface. It *almost* looks like your favourite imageboard!

And that's not all!

## Why Rust?

Just for fun. This is a learning project after all. The author of the original project wrote it on C#, the language which I'm familiar with. It's much more interesting to try something new, isn't it?


# To-do list
Since it's a learning and rewriting project, there's still much to implement:

## Core parts
- [x] PNG steganography tool
- [ ] Posts database with legacy (old nanoboard project) database support
- [ ] Handling of captcha, POW and posts signature
- [ ] Nanoposts encoding and decoding

## Utility parts
- [ ] Server with endpoints to get nanoposts
- [ ] Frontend