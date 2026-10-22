# Third-party notices

Veyra: Dominion is proprietary (see `LICENSE`). It is built with the components below, which keep their own licences. Full texts are in `licenses/` and ship next to the executable.

## Fonts

| Font | Author | Licence | Text |
|---|---|---|---|
| Jacquard 24 | Sarah Cadigan-Fried (The Soft Type Project Authors) | SIL OFL 1.1 | `licenses/fonts/OFL-Jacquard24.txt` |
| Jersey 15 | Sarah Cadigan-Fried (The Soft Type Project Authors) | SIL OFL 1.1 | `licenses/fonts/OFL-Jersey15.txt` |
| Press Start 2P | Cody "CodeMan38" Boisclair (The Press Start 2P Project Authors) | SIL OFL 1.1 | `licenses/fonts/OFL-PressStart2P.txt` |

## Rust crates

48 packages from `Cargo.lock` (macroquad and every transitive dependency, all target platforms). None is copyleft. Where a crate is offered under several licences, the one marked *(chosen)* is the one this game uses; MIT is preferred when offered. Texts in `licenses/crates/<name>-<version>/`.

| Package | Version | Offered | Used | Copyright | Notes |
|---|---|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | MIT (chosen) | Jonas Schievink, oyvindln (no copyright line in the licence file) |  |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | MIT (chosen) | Zakarum (no copyright line in the licence file) |  |
| audir-sles | 0.1.0 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2020 msiglreith |  |
| audrey | 0.3.0 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2016 RustAudio Developers |  |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | MIT (chosen) | Copyright (c) 2018 Josh Stone |  |
| bitflags | 1.3.2 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) 2014 The Rust Project Developers |  |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | MIT (chosen) | Copyright (c) 2019 Daniel "Lokathor" Gee. |  |
| byteorder | 1.5.0 | Unlicense OR MIT | MIT (chosen) | Copyright (c) 2015 Andrew Gallant |  |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2014 Alex Crichton |  |
| color_quant | 1.1.0 | MIT | MIT | Copyright (c) 2016 PistonDevelopers |  |
| core_maths | 0.1.1 | MIT | MIT | Copyright (c) 2024 Robert Bastian |  |
| crc32fast | 1.5.2 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2018 Sam Rijs, Alex Crichton and contributors |  |
| dasp_frame | 0.11.0 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) mitchmindtree | MIT text generated: the published crate ships no MIT licence file. |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) mitchmindtree | MIT text generated: the published crate ships no MIT licence file. |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | MIT (chosen) | Copyright (c) 2016--2023 |  |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | MIT (chosen) | The image-rs Developers (no copyright line in the licence file) |  |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2014-2026 Alex Crichton |  |
| foldhash | 0.1.5 | Zlib | Zlib | Copyright (c) 2024 Orson Peters |  |
| fontdue | 0.9.4 | MIT OR Apache-2.0 OR Zlib | MIT (chosen) | Copyright (c) 2019 Joe C (mooman219) | Also includes licences of vendored code (`vendored/`). |
| glam | 0.27.0 | MIT OR Apache-2.0 | MIT (chosen) | Cameron Hart (no copyright line in the licence file) |  |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2016 Amanieu d'Antras |  |
| hound | 3.5.1 | Apache-2.0 | Apache-2.0 | Ruud van Asseldonk (no copyright line in the licence file) |  |
| image | 0.24.9 | MIT OR Apache-2.0 | MIT (chosen) | The image-rs Developers (no copyright line in the licence file) |  |
| lewton | 0.9.4 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) 2016 est31 <MTest31@outlook.com> and contributors |  |
| libc | 0.2.191 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) The Rust Project Developers |  |
| libm | 0.2.16 | MIT | MIT | Copyright (c) 2018 Jorge Aparicio; Copyright © 2005-2020 Rich Felker, et al.; Copyright © 1993,2004 Sun Microsystems or; Copyright © 2003-2011 David Schultz or; Copyright © 2003-2009 Steven G. Kargl or; Copyright © 2003-2009 Bruce D. Evans or; Copyright © 2008 Stephen L. Moshier or; Copyright © 2017-2018 Arm Limited |  |
| macroquad | 0.4.16 | MIT OR Apache-2.0 | MIT (chosen) | not-fl3 (no copyright line in the licence file) |  |
| macroquad_macro | 0.1.8 | MIT/Apache-2.0 | MIT (chosen) | not-fl3 (no copyright line in the licence file) |  |
| malloc_buf | 0.0.6 | MIT | MIT | Copyright (c) Steven Sheldon | MIT text generated: the published crate ships no MIT licence file. |
| maybe-uninit | 2.0.0 | Apache-2.0 OR MIT | MIT (chosen) | Copyright (c) 2010 The Rust Project Developers |  |
| miniquad | 0.4.11 | MIT OR Apache-2.0 | MIT (chosen) | not-fl3 (no copyright line in the licence file) |  |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | MIT (chosen) | Copyright 2013-2014 RAD Game Tools and Valve Software; Copyright 2010-2014 Rich Geldreich and Tenacious Software LLC; Copyright (c) 2017 Frommi; Copyright (c) 2017-2024 oyvindln |  |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | MIT (chosen) | Copyright 2013-2014 RAD Game Tools and Valve Software; Copyright 2010-2014 Rich Geldreich and Tenacious Software LLC; Copyright (c) 2017 Frommi; Copyright (c) 2017-2024 oyvindln |  |
| ndk-sys | 0.2.2 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) The Rust Windowing contributors | MIT text generated: the published crate ships no MIT licence file. |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2014 The Rust Project Developers |  |
| objc-rs | 0.2.8 | MIT | MIT | Copyright (c) Steven Sheldon |  |
| ogg | 0.7.1 | BSD-3-Clause | BSD-3-Clause | Copyright (c) 2016-2017 est31 <MTest31@outlook.com> and contributors; Copyright (c) 2002-2015 Xiph.org Foundation |  |
| png | 0.17.16 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2015 nwin |  |
| quad-alsa-sys | 0.3.2 | MIT | MIT | Copyright (c) 2018 diwic |  |
| quad-rand | 0.2.3 | MIT | MIT | Copyright (c) not-fl3 | MIT text generated: the published crate ships no MIT licence file. |
| quad-snd | 0.2.8 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) Fedor Logachev | MIT text generated: the published crate ships no MIT licence file. |
| simd-adler32 | 0.3.10 | MIT | MIT | Copyright (c) [2021] [Marvin Countryman] |  |
| smallvec | 0.6.14 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) 2018 The Servo Project Developers |  |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | MIT (chosen) | Copyright (c) 2018 Yevhenii Reizner |  |
| winapi | 0.3.9 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) 2015-2018 The winapi-rs Developers |  |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) Peter Atashian | MIT text generated: the published crate ships no MIT licence file. |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | MIT (chosen) | Copyright (c) Peter Atashian | MIT text generated: the published crate ships no MIT licence file. |
| zlib-rs | 0.6.8 | Zlib | Zlib | (C) 2024 Trifecta Tech Foundation |  |
