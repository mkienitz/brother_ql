# Changelog

All notable changes to this project will be documented in this file.

## [3.0.0] - 2026-03-20

### Bug Fixes
- [**breaking**] Correctly pass print job builder arguments to print job ([3db3df7](https://github.com/mkienitz/brother_ql/commit/3db3df7ea1664f0f5d5c1a106935d70f6bc6a92e))


### Documentation
- Update code examples in the README to use current API ([041d6c6](https://github.com/mkienitz/brother_ql/commit/041d6c696b5593ed5846edd736aac82e8702d5f4))
- Streamline examples ([25618c7](https://github.com/mkienitz/brother_ql/commit/25618c7bda1c333de6342782547749801aaf0d70))


### Features
- [**breaking**] `CutEvery(n)` cut behavior now takes `NonZeroU8` for correctness ([54af137](https://github.com/mkienitz/brother_ql/commit/54af137f08c9442145dc1045ab42c642df622b81))
- [**breaking**] `no_copies` of a print job must be populated with `NonZeroU8` ([c33976f](https://github.com/mkienitz/brother_ql/commit/c33976fa2974c31ec2c22d634dd637a0877ec0ee))
- Add helper to infer media type from fields returned in printer status information ([97fa286](https://github.com/mkienitz/brother_ql/commit/97fa286bf0d6ce99df972ea4830dccfcf8a20e39))
- Impl `Display` for printer model names, including dashes ([4d04c75](https://github.com/mkienitz/brother_ql/commit/4d04c75fa2d60ff7618ffbd4aa1b3a83ca268212))
- Impl `Display` for StatusInformation struct ([ac1ea31](https://github.com/mkienitz/brother_ql/commit/ac1ea3157a0d6b669c70e0adbba72fb32f191f32))
- Add common derived traits to status-related types ([94c3005](https://github.com/mkienitz/brother_ql/commit/94c3005e794e368496103a9e181f87b5f623fa4c))


### Miscellaneous
- [**breaking**] Fix typos in "occurred" ([60513f9](https://github.com/mkienitz/brother_ql/commit/60513f9e378519aa4825d9f0d0c4a7ee9b649578))
- Update dependencies ([c9d31e1](https://github.com/mkienitz/brother_ql/commit/c9d31e1e74cfae8d88d4626038bc92c8ef5d8468))


### Performance
- Use a flat vector in the raster command builder ([cd047eb](https://github.com/mkienitz/brother_ql/commit/cd047eb547ec3658ce63162a9d3bcec208f50cf2))


### Refactoring
- Pass arguments by value for `validate_status` ([b040bc2](https://github.com/mkienitz/brother_ql/commit/b040bc2efcd524dfed358d180b872dddf363f5b8))
- [**breaking**] Rename the `CutBehavior::None` option to `::NoCut` ([99685c6](https://github.com/mkienitz/brother_ql/commit/99685c60ed7542b15b57800cdab7960592591bbc))
- [**breaking**] Streamline the print job creation API to always use `PrintJobBuilder` ([a5113dd](https://github.com/mkienitz/brother_ql/commit/a5113dda42c4690ddc1c6cd5af5bae97ee2adebf))


### Testing
- Add unit tests for print jobs ([18e5007](https://github.com/mkienitz/brother_ql/commit/18e5007b0b4aef0a62be36195b5606ab1d79dcfc))
- Add additional test coverage ([ae23179](https://github.com/mkienitz/brother_ql/commit/ae231791f43c7b1adec4d0330665179afa9ee2e8))

