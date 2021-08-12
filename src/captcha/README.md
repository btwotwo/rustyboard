# Disclaimer
This is **not** an original implementation. It was taken from the previous Nanoboard Project. The pregenerated captchas were also made by previous developers.

# What is nanoboard captcha?
In order to add some basic protection from spammenrs a captcha system was implemented. It consists of two parts: **Captcha calulation** and **Proof-of-work**.

## Captcha calucation
All captchas in the nanoboard project are pregenerated and stored inside the `captcha.nbc` file. Each captcha is 189 bytes long. It consists from:

1) 32 bytes of `ed25519` public key
2) 32 bytes seed (will be explained later)
3) 125 bytes of `50x20` 1-bit image. 

### Seed ~~and feed~~
Seed is a base for `ed25519` private key. In order to get `decrypted_seed`, you need to:
1) Compute `hash = SHA512(answer + public_key)`
2) `XOR` every nth byte of `encrypted_seed` with every `nth & 63`th byte of `hash`

After that you can verify that user guessed captcha correctly. To do that, you'll transform a seed into a `private_key`, then sign 1 byte with it, and verify the signature with `public_key`. If the signature is valid, then consider that the captcha was solved correctly.

### Captcha image
