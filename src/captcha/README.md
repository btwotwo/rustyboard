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
1) Compute `hash = SHA512(UTF8(answer + public_key as hexstring))`
2) `XOR` every nth byte of `encrypted_seed` with every `nth & 63`th byte of `hash`

After that you can verify that user guessed captcha correctly. To do that, you'll transform a seed into a `private_key`, then sign 1 byte with it, and verify the signature with `public_key`. If the signature is valid, then consider that the captcha was solved correctly.

If the captcha was solved, then we need to add a signature to the user's post.
It is calculated like this: `ed25519.sign(UTF8(post), private_key)`. After the hash is added, post should look like this:
`post_message[pow=abcedf...][sign=abcdef...]`

### Captcha image
Each byte converted to bits. If bit is `1`, then color is white, otherwise it's black. Image is filled column-wise (starting from `(x, y) = (0, 0), (0, 1), (0, 2)...(0, 19); (1, 0), (1, 2), etc...`)

## Proof-of-work
Every post should have hash. Hash calculates from `UTF8(post message + 128 random bytes)`.
Until there's **three consecutive zeros starting from the 3rd byte**, new 128 random bytes are generated and hash is recalculated.

**First three bytes** of generated hash are the index of the captcha. It is calculated like this: `hash[0] + (hash[1] * 256) + (hash[2] * 256)) % (captcha_file_size / captcha_block_length)`. 

Hash of the post isn't stored, only random bytes.