Rust-based atmospherics for Space Station 13 using [byondapi](https://github.com/spacestation13/byondapi-rs).

The compiled binary on Citadel is compiled for Citadel's CPU, which therefore means that it uses [AVX2 fused-multiply-accumulate](https://en.wikipedia.org/wiki/Advanced_Vector_Extensions#Advanced_Vector_Extensions_2).

Binaries in releases are without these optimizations for compatibility. But it runs slower and you might still run into issues, in that case, please build the project yourself.

You can build auxmos like any rust project, though you're gonna need `clang` version `6` or more installed. And `LIBCLANG_PATH` environment variable set to the bin path of clang in case of windows. Auxmos only supports `i686-unknown-linux-gnu` or `i686-pc-windows-msvc` targets on the build.

Use `cargo t generate_binds` to generate the `bindings.dm` file to include in your codebase, for the byond to actually use the library, or use the one on the repository here (generated with feature `katmos`).

The `master` branch is to be considered unstable; use the releases if you want to make sure it actually works. [The latest release is here](https://github.com/Putnam3145/auxmos/releases/latest).

---

## BlueMoon-Station fork

Этот форк добавляет фичу **`bluemoon_reactions`**: **все** атмосферные реакции BlueMoon-Station перенесены в Rust (`src/reaction/bluemoon.rs`) и считаются там: огонь (плазма, тритий, generic), синтез (fusion), ноблиум (подавление, образование), водяной пар, нитрил, BZ, стимулум, стерилизация миазмы, разложение NO, Hagedorn/dehagedorn, фреон (горение, образование), галон, хиллий, заукер, нитрий, плуоксий, прото-нитрат (все три реакции), антиноблиум. Газы регистрируются по-прежнему в DM; константы — в `src/gas/constants.rs`.

**Опциональные прок в DM** (если не определить, вызов просто игнорируется или не делается):
- `/proc/bluemoon_add_research_points(amount)` — для начисления очков исследований (BZ, стимулум, ноблиум, миазма, Hagedorn). Пример: `proc/bluemoon_add_research_points(amount) { SSresearch?.science_tech?.add_point_type(TECHWEB_POINT_TYPE_DEFAULT, amount) }`
- На тайле (holder) при реакции фреона можно определить `bluemoon_freon_hot_ice_check(air)` и внутри проверить температуру 120–160 K и `prob(5)`, затем создать hot_ice. Иначе в Rust вызывается этот callback только если он есть; генерация hot_ice остаётся на стороне DM.

### Сборка для BlueMoon-Station

```bash
cargo build --release --features "bluemoon_reactions"
```

Под Windows нужен `clang` (например, LLVM) и переменная `LIBCLANG_PATH`. Цель: `i686-pc-windows-msvc` (32-bit) или ваша целевая архитектура по умолчанию.

Биндинги для BYOND генерируйте с той же фичей:

```bash
cargo run --features "bluemoon_reactions" -- generate_binds
```

(или используйте существующий `bindings.dm` из основного репозитория auxmos — имена процедур совместимы; важно собрать `.dll`/`.so` с `bluemoon_reactions`.)
