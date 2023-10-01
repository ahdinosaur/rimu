# Changelog

with help from [`git log`](https://www.git-scm.com/docs/git-log):

```shell
git log --oneline --format="- [%h](https://github.com/ahdinosaur/rimu/commit/%H): %s"
```

## 0.2.0

- [ae2929c](https://github.com/ahdinosaur/rimu/commit/ae2929c5635a96547144fb2c9762b8ce3405930f): Meta dev (#67)
- [a39b055](https://github.com/ahdinosaur/rimu/commit/a39b0559540edfdb3b72d5daaeab980f03da71bc): Native functions and a standard library (#65)
- [5fdc3e5](https://github.com/ahdinosaur/rimu/commit/5fdc3e5e0265d608edd8a867595391e7935185e7): Another pass on the docs and playground (#64)
- [6e42351](https://github.com/ahdinosaur/rimu/commit/6e423516d2e69d289fa4f16b919e496cbd9d5757): Support function closures (#62)
- [2b5553c](https://github.com/ahdinosaur/rimu/commit/2b5553cbdbdb2ba4b9bc012b00cb818075529220): remove wee_alloc, use standard allocator
- [b5569d9](https://github.com/ahdinosaur/rimu/commit/b5569d9d1006024081fc81ccbdc474a4bf4395ed): remove leftover file
- [ce6066e](https://github.com/ahdinosaur/rimu/commit/ce6066ee018f1d85a354db258a152ca96b78f48d): Lezer syntax highlighting (#59)
- [3a47beb](https://github.com/ahdinosaur/rimu/commit/3a47bebaf15026dc13ccb6138c222ba80effee66): Wake up babe new syntax just dropped (#58)
- [22726d2](https://github.com/ahdinosaur/rimu/commit/22726d2af65674d158c0546f491195cbd45b098d): Fun fun functions (#57)
- [021664d](https://github.com/ahdinosaur/rimu/commit/021664dd7b427ddf4400e18ba58f983e687e9142): fix block operations error (#56)

## 0.1.0

- [e28fe9f](https://github.com/ahdinosaur/rimu/commit/e28fe9f41c4a77e18f46e8015d04afd2eb3cba40): rename packages, prep for crates publish
- [114d390](https://github.com/ahdinosaur/rimu/commit/114d390bd9d7db2cf9e9c582071eec31178521d0): First pass at documentation site (#25)
- [26ff66b](https://github.com/ahdinosaur/rimu/commit/26ff66b1979a78be3ff819151da38f63d75d7973): update theme and code without a codemirror reboot (#52)
- [089b5c9](https://github.com/ahdinosaur/rimu/commit/089b5c9d9b2e0ea4329697c761f963bc70b3f2a1): update GitHub issue templates (#45)
- [f07c7b2](https://github.com/ahdinosaur/rimu/commit/f07c7b21bfda402f1fe90145ec99be038a51e5e9): add Playground web app (#41)
- [12fac9a](https://github.com/ahdinosaur/rimu/commit/12fac9af148dafa1bc1d8eccaa64480d6183f2dd): change from `BTreeMap` to `IndexMap`: preserve order (#40)
- [4fcb2db](https://github.com/ahdinosaur/rimu/commit/4fcb2db37672af70dccbde9060e8f83a0cde6779): reorganize crates (#38)
- [212baf5](https://github.com/ahdinosaur/rimu/commit/212baf5adae080d884c17a68ed5c73b3d847f06c): truthiness: everything except `false` and `null` is truthy (#37)
- [f2f8f3a](https://github.com/ahdinosaur/rimu/commit/f2f8f3a07e9b62402b00ab40d871da1d5460e5f6): add command-line interface (#36)
- [620af7c](https://github.com/ahdinosaur/rimu/commit/620af7c4dc6494954dc04013f23b587a70dd2c23): finish Block: re-write parser, add evaluator, and replace main lib (#30)
- [d7e8c69](https://github.com/ahdinosaur/rimu/commit/d7e8c6900e0c0ffc4698c4e8eaab1c789867f548): Blocks: lexer (with indents) and parser (#28)
- [837833c](https://github.com/ahdinosaur/rimu/commit/837833c192cf4b82a833007b2d91537dcfbc3197): Revert "rename operations to blocks" (#27)
- [c27213c](https://github.com/ahdinosaur/rimu/commit/c27213c6b37603f4d46f37b9fdda186f8247d7cc): repl errors (#19)
- [331c911](https://github.com/ahdinosaur/rimu/commit/331c9110a00553076fbf7c6ca9824fb033a8e0d7): Expression evaluator and repl (#15)
- [e6bf713](https://github.com/ahdinosaur/rimu/commit/e6bf7133a73af0505efc91ad638dccb29f0dc7b4): Expressions: lexer and parser (#11)
- [d605854](https://github.com/ahdinosaur/rimu/commit/d605854228fcc2e8d4389d50730b9b45edb64a6e): rename operations to blocks (#10)
- [964177c](https://github.com/ahdinosaur/rimu/commit/964177cbdd35b1e19f75efbceae5d29e85fd3f25): add If operation (#8)
- [5926af4](https://github.com/ahdinosaur/rimu/commit/5926af48b3d7a5afc1406bf8693dfa8a96c0ab00): add Let operation (#7)
- [3130490](https://github.com/ahdinosaur/rimu/commit/31304902e3758132764e67ccfa33a745868e8083): split into two crates: `rimu` and `rimu-value` (#1)

## 0.0.1

- [7cdf5f1](https://github.com/ahdinosaur/rimu/commit/7cdf5f183ec79e819de7999e9e138138297d6dac): in the beginning
