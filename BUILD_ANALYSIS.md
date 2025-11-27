# Build Analysis & Optimization Opportunities

## Current Build Summary

### Binaries & Sizes (Release Builds)

| Binary | Size | Build Time | Key Dependencies |
|--------|------|------------|------------------|
| `cli` (TUI) | 1.1 MB | 14.66s | ratatui, clap, crossterm |
| `app` (GUI) | 4.3 MB | 25.27s | egui, eframe, glutin, arboard |

### Dependency Breakdown

#### CLI (`crates/cli`)
- **TUI Framework**: `ratatui` + `crossterm` (signal handling, terminal control)
- **CLI Parsing**: `clap` with derive feature
- **Shared**: lib (pcap-parser, serde, etc.)

**Total deps**: ~60 transitive

#### App (`crates/app`)
- **GUI Framework**: `egui` + `eframe` + `glutin` (OpenGL context & window)
- **Web Support**: `wasm-bindgen`, `web-sys`, `js-sys` (conditional for WASM)
- **Desktop Features**: `rfd` (file dialog), `env_logger` (optional)
- **UI Extras**: `egui_json_tree`, `arboard` (clipboard)
- **Platform Bindings**: Multiple `objc2-*` for macOS, `url` for URL parsing
- **Images**: `image`, `png` crates (fonts, icons)

**Total deps**: ~120+ transitive

---

## Optimization Opportunities

### 1. **CLI Binary (1.1 MB) - Moderate Optimization Potential**

#### Current Size Culprits
- `ratatui`: Large TUI framework (~100KB+ when compiled)
- `clap` with `derive` feature: Proc-macro overhead
- `crossterm`: Signal handling + termios bindings for all platforms
- All `pcap-parser` + `nom` parsing logic

#### Strategies (Ranked by Impact)

**High Impact:**
- **Switch to lighter CLI parser**: Replace `clap` with `lexopt` (minimal) or `argh` (Google's)
  - Estimated savings: ~100-200KB
  - Trade-off: Lose derive macro convenience, manual parsing
  
- **Replace ratatui with lighter TUI**: Use `cursive` or even simpler `tui-rs` alternatives
  - Current ratatui pulls in: `strum`, `darling`, `instability` (macro framework), `lru`, `unicode-*`
  - Estimated savings: ~50-100KB
  - Trade-off: Less feature-rich, may need custom rendering

- **Reduce pcap-parser scope**: Create minimal feature set for just UDP packets
  - `pcap-parser` pulls in `nom` + `circular` parser combinator
  - Estimated savings: ~50KB (if custom parser)
  - Trade-off: More maintenance burden

**Medium Impact:**
- **Strip symbols**: Already done in release profile (`strip = true`)
- **Enable LTO aggressively**: Consider `lto = "fat"` for CLI (slower build, smaller binary)
  - Estimated savings: ~5-10%
  
- **Reduce CLI features**: Make filter/search optional features
  - Current filtering uses `serde_json` traversal (included by default)
  - Estimated savings: ~20KB if made optional

---

### 2. **App/GUI Binary (4.3 MB) - Significant Optimization Potential**

#### Current Size Culprits
- `egui` + rendering stack: ~500KB+ (fonts, UI logic, rendering)
- `egui-winit` + `glutin`: Window/context management (~300KB+)
- `arboard` + clipboard dependencies: Platform-specific bindings (~100KB+)
- `url` parsing chain (IDNA, Unicode normalization): ~200KB+
- Image crate + PNG decoder: ~150KB+
- `objc2-*` bindings for macOS: ~100KB+

#### Strategies (Ranked by Impact)

**High Impact:**
- **Make clipboard optional**: `arboard` can be a feature flag
  - Estimated savings: ~100KB
  - Trade-off: No copy-paste support without feature
  
- **Replace or slim egui fonts**: `egui_default_fonts` embedded entire TTF
  - Estimated savings: ~50KB
  - Trade-off: Limited font options
  
- **Conditionally include image support**: Only needed for desktop, not WASM
  - Move image crate to desktop-only feature
  - Estimated savings: ~150KB+
  - Trade-off: Can't display embedded images in web version

- **Minimize Unicode/URL handling**: Most URL parsing is overkill for PCAP viewer
  - Could use simpler custom URL parser
  - Estimated savings: ~150-200KB
  - Trade-off: Less RFC-compliant, but fine for internal use

**Medium Impact:**
- **Desktop-only profile**: Create separate release profiles
  - `cargo build --release -p app --features desktop` uses full eframe
  - `cargo build --release --target wasm32 -p app` uses lean WASM version
  - Can apply different optimization passes
  
- **Separate web crate**: Split `app` into `app-desktop` + `app-web`
  - Prevents unnecessary deps in web build (glutin, arboard, etc.)
  - Build profiles become simpler
  - Estimated savings: ~500KB in web build

- **Use simpler JSON viewer**: Replace `egui_json_tree` with minimal custom widget
  - Estimated savings: ~50KB

---

### 3. **Build Time Optimization**

#### Current Times
- CLI: 14.66s (clean build)
- App: 25.27s (clean build)

#### Opportunities

**Fast Rebuild Cycle:**
- Use `cargo-check` for development (`cargo check -p cli`)
- Use `--lib` to skip binary builds during lib development
- Consider `mold` linker on Linux (faster linking)

**Profile Optimization:**
```toml
# Add to Cargo.toml profile.release
codegen-units = 1      # Single codegen unit (current)
lto = true             # Link-time optimization (current)

# Could add for faster builds (slightly larger binary):
# lto = "thin"       # Faster LTO
# opt-level = 2      # Instead of "s" (size)
```

**Parallel Builds:**
- Workspace with separate crates allows better parallelization
- `cargo build -p lib -p cli` builds in parallel
- Already good structure

---

## Recommended Action Plan

### Phase 1: Quick Wins (Low Risk)
1. **Create feature flags for non-essential deps**
   - Make `egui_json_tree` optional
   - Make `arboard` (clipboard) optional in app
   - Estimated savings: ~20KB for CLI, ~150KB for App

2. **Separate web from desktop builds**
   - Create distinct features that prevent cross-platform deps
   - Move image crate to desktop-only
   - Estimated savings: ~200KB in WASM builds

### Phase 2: Medium Optimization (Moderate Refactoring)
3. **Evaluate CLI parser alternatives**
   - Create branch with `lexopt` instead of `clap`
   - Measure binary size impact
   - Estimated savings: ~100KB, build time neutral

4. **Create slim desktop profile**
   - Different optimization flags for desktop vs web
   - Use `--profile` instead of `release`
   - Build as dependency in xtask

### Phase 3: Structural Changes (Major Refactoring)
5. **Consider app split into desktop + web crates**
   - Prevents cross-platform dependency bloat
   - Allows independent feature sets
   - Estimated savings: ~500KB in web, 0KB in desktop
   - Time investment: ~2-4 hours

6. **Evaluate TUI framework replacement** (for CLI only)
   - Consider `cursive` or minimal alternatives
   - Complex refactoring, measure first
   - Estimated savings: ~50-100KB
   - Time investment: ~4-8 hours

---

## Feature Flags to Add

### lib/Cargo.toml
```toml
[features]
default = ["full"]
full = ["pcap-parser", "base64", "hex"]
minimal = []  # Parse only, no encoding
```

### cli/Cargo.toml
```toml
[features]
default = ["tui"]
tui = ["ratatui", "crossterm"]
minimal = []  # Just CLI parsing
search = ["default"]  # Full search/filter features
```

### app/Cargo.toml
```toml
[features]
default = []
desktop = ["rfd", "env_logger", "clipboard"]
web = ["wasm-bindgen", "web-sys", "js-sys"]
clipboard = ["arboard"]  # Optional clipboard
gui = ["egui", "eframe"]  # All GUI features
minimal-gui = ["gui"]  # Without clipboard/fonts
json-viewer = ["egui_json_tree"]
```

---

## Build Time Analysis

### Why app is slower (25.27s vs 14.66s for CLI)

1. **More dependencies**: 120+ transitive vs 60+
2. **Larger crates**: `egui` compilation is heavy (~8s alone)
3. **Platform-specific code**: macOS bindings add complexity
4. **Image processing**: PNG decoder setup
5. **Linker time**: 4.3MB binary takes longer to link

### Potential improvements
- Split web from desktop → **2-3s faster web build**
- Replace clap in CLI → **1-2s faster CLI build**
- Reduce features by default → **2-4s faster full build**

---

## Current Profile Analysis

### Workspace Profile (Cargo.toml)
```toml
[profile.release]
lto = true           # ✅ Enabled (good for size)
opt-level = "s"      # ✅ Size optimized (vs "z" for WASM)
codegen-units = 1    # ✅ Single unit (good for size, slower build)
strip = true         # ✅ Symbols stripped
```

**Assessment**: Already well-optimized for size. Further gains require dependency changes.

### WASM-Specific Profile
```toml
[profile.release-wasm]
inherits = "release"
opt-level = "z"      # Even smaller than "s"
panic = "abort"      # Smaller panic handler
```

**Assessment**: Good, but only used if xtask web --small is called. Should be default for web.

---

## Summary Table

| Optimization | CLI Impact | App Impact | Effort | Risk |
|--------------|-----------|-----------|--------|------|
| Feature flags (non-essential) | -20KB | -150KB | 1 hour | Low |
| Separate web crate | N/A | -500KB (web only) | 3 hours | Low |
| CLI parser replacement | -100KB | N/A | 1 hour | Low |
| TUI framework replacement | -50KB | N/A | 6 hours | Medium |
| More aggressive LTO | -5-10% | -5-10% | 0.5 hours | Low |
| Image crate optional | -50KB | -150KB (desktop) | 1 hour | Low |
| URL parser replacement | N/A | -150KB | 2 hours | Medium |

**Best ROI**: Feature flags + Separate web crate → ~650KB saved in web builds, minimal effort.
