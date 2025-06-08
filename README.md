> [!WARNING]
> ``dncli`` is still in alpha, expect bugs, errors, or anything.

# dncli
dncli ("**d**ow**n**load **cli**ent") is a download client, like wget, written in Rust.

![Logo of dncli (looks shit ik)](/images/Untitled4_20250608210312.png)
![enbyware (ENBYYYYY NONBINARY RAHHHHHHGUFHFHFHFHFHFGGHFHG kinoite is non-binary btw)](https://camo.githubusercontent.com/bee100e0a2439d329fd52512a3acd1b4df3adf924e315507c5433d68ab79ab95/68747470733a2f2f70726964652d6261646765732e706f6e792e776f726b6572732e6465762f7374617469632f76313f6c6162656c3d656e627977617265266c6162656c436f6c6f723d2532333535352673747269706557696474683d3826737472697065436f6c6f72733d464346343334253243464646464646253243394335394431253243324332433243)
The aim of dncli is to be more 
performant, faster and reliable, 
Unlike wget, it still relies on Rust 
dependencies that need to be 
installed thru cargo, Rust's package 
manager, dncli will hopefully become 
fully independent from those 
dependencies soon enough (no offence).

## Erm... how do I install this..?

Good question little user!, you first 
clone this Github repository with ``git`` or ``gh`` with:

```
git clone https://github.com/kinoite/dncli.git
```
or:
```
gh repo clone kinoite/dncli
```

After that, go into the cloned repository, then run:
```
cargo build --release
```

If you want, you can move the dncli binary to your $PATH, If you do not 
know what that is, it is essentially an environment variable that tells the shell where to look for
executable files when you type a command, It should be something like /
usr/bin, /usr/local/bin, /bin or more/other.

In this guide, we are gonna move it 
to /usr/local/bin, but you can set it to any $PATH you desire 
(though i dont recommend putting it in /bin)
```
sudo mv target/release/dmcli /usr/local/bin
```

And... that's pretty much it!, dncli is now installed-

## But like- how do I use it!?

Calm down a bit, heh, anyways-

To download something from the web 
with dncli, we're gonna have to use the ``-u`` (``--url``) flag, 
for example:

```
dncli -u https://rustacean.net/assets/cuddlyferris.png
```

It'll download in the speed of light!, 
or- somewhere close, just see for yourself, fellow user!-

## WHAT SYSTEM DOES THIS WORK ON??!??!?
Woah... calm down buckaroo, not sure 
why you're this despera- ANYWAYS!

dncli works on any system that 
supports Rust, or cargo and crates.io packages, atleast.

## W-Where do I ask questions?
Make a pull request!, I'll answer when I can!~ hehe!

## Oh... I FOUND A BUG CODE RED CODE RED AAAAAAGHHHH!!!!
WOAH!, You scared me please calm down...
Anyways- if you have found a bug within dncli, Make a new issue, and 
explain what happened and try how to reproduce the bug 
(so i can see it myself)

I'll try to help you as much as I can!

## How do I c-contribute??
You can submit pull requests, and send what can be changed in the code and what should be dropped, I'd love contributors! since I never had any contributors before, **but you don't have to contribute!**
