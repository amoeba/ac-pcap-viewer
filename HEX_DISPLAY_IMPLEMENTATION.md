# Hex Display Implementation

## Overview

This document describes the implementation of intelligent hex/decimal display in the AC PCAP Parser's tree view, along with click-to-filter functionality.

## What Was Implemented

### 1. Custom AC-Aware JSON Tree Viewer

**File:** `crates/app/src/ui/ac_json_tree.rs`

A completely new custom tree viewer that:
- Replaces the generic `egui_json_tree` for message/packet detail display
- Understands the AC protocol and displays values appropriately
- Provides click-to-filter functionality on all values

### 2. Intelligent Field Type Detection

The viewer automatically determines whether a field should be displayed as hex or decimal based on:

#### Field Names (Always Hex)
- `ObjectId`, `object_id`
- `OpCode`, `opcode`
- `Flags`, `flags`
- `SpellId`, `spell_id`
- `Id` (for spell IDs, enchantment IDs, etc.)
- `CasterId`, `caster_id`
- `Sequence`, `sequence`
- `ItemType`, `item_type`
- `DamageType`, `damage_type`
- `ContainerId`, `container_id`
- `WielderId`, `wielder_id`
- `TargetId`, `target_id`
- `Key` (property keys)
- Any field ending with `Id` or `_id`
- Any field containing `Sequence`

#### Value-Based Detection (Auto Mode)
- Numbers > 255 are shown in hex with decimal in parentheses
- Numbers ≤ 255 are shown in decimal only
- Large numbers (> 0xFFFF) are assumed to be IDs and shown in hex

### 3. Display Formats

**Hex Format:**
```
0xF7B0 (63408)
```
- Primary display: `0xF7B0` (uppercase hex with 0x prefix)
- Secondary display: `(63408)` (decimal in parentheses, grayed out)

**Decimal Format:**
```
42
```
- Simple decimal number

**String Hex Format:**
```
0xF7B0 (63408)
```
- For string values that are hex (like OpCode field "F7B0")
- Automatically detects hex strings and adds 0x prefix

### 4. Click-to-Filter

All values in the tree are clickable:
- **Visual feedback:** Values shown in hyperlink color
- **Cursor feedback:** Pointing hand cursor on hover
- **Tooltip:** "Click to filter" on hover
- **Behavior:** Clicking a value sets the search query to that value
- **Works with existing filter:** The existing hex/decimal filter system works seamlessly

Example workflow:
1. User views a message in the tree view
2. Sees `ObjectId: 0x80414B6A (2151762794)`
3. Clicks on the value
4. Search box updates to `0x80414B6A`
5. Message list filters to show only messages with that ObjectId
6. User can also manually type `2151762794` (decimal) and it will find the same messages

### 5. Visual Enhancements

- **Hyperlink color:** Clickable values use the UI's hyperlink color (typically blue)
- **Pointer cursor:** Hand cursor indicates clickability
- **Hover tooltip:** "Click to filter" message
- **Grayed decimal:** Secondary decimal values shown in smaller, grayed text
- **Color coding:** Booleans use green (true) / red (false)

## Code Structure

### `AcJsonTree` struct
```rust
pub struct AcJsonTree {
    id: String,                      // Unique ID for tree state
    expanded_paths: HashSet<String>, // Track which paths are expanded
    response: TreeResponse,          // Return interaction results
}
```

### `DisplayFormat` enum
```rust
pub enum DisplayFormat {
    Decimal,  // Show as decimal only
    Hex,      // Show as hex with decimal in parens
    Auto,     // Decide based on value size
}
```

### `TreeResponse` struct
```rust
pub struct TreeResponse {
    pub filter_clicked: Option<String>, // Value to filter by, if clicked
}
```

### Key Methods

**`determine_format(key, value)`**
- Analyzes field name and value
- Returns appropriate DisplayFormat
- Uses heuristics based on AC protocol knowledge

**`show_number_value(ui, num, format)`**
- Displays numeric value with appropriate format
- Handles click detection
- Returns clicked value for filtering

**`is_hex_string(s)`**
- Checks if a string is a hex value (e.g., "F7B0")
- Validates all characters are hex digits
- Length limited to 16 chars to avoid false positives

## Integration Points

### Detail Panel (`crates/app/src/ui/detail_panel.rs`)

**Before:**
```rust
JsonTree::new(&tree_id, &app.messages[idx].data)
    .default_expand(egui_json_tree::DefaultExpand::ToLevel(1))
    .show(ui);
```

**After:**
```rust
let response = AcJsonTree::new(&tree_id).show(ui, &app.messages[idx].data);
if let Some(value) = response.filter_clicked {
    filter_value = Some(value);
}

// ... later ...
if let Some(value) = filter_value {
    app.search_query = value;
}
```

## Filter Compatibility

The click-to-filter feature works seamlessly with the existing filter system (`crates/app/src/filter.rs`):

**Existing Filter Features:**
- Parse hex input: `0xF7B0` → filters for both hex and decimal representations
- Parse decimal input: `63408` → filters for both hex and decimal representations
- String matching: Case-insensitive substring search

**New Click-to-Filter:**
- Clicking `0x80414B6A` sets search to `0x80414B6A`
- Filter system automatically matches both hex and decimal forms
- Works for any field value: IDs, opcodes, strings, numbers

## Examples

### Message Tree Display

```
ObjectId: 0x80414B6A (2151762794)     [clickable, blue]
OpCode: 0xC9 (201)                    [clickable, blue]
Type: "Item_SetAppraiseInfo"          [clickable, blue]
Properties:
  ▶ IntProperties
    Dyable: 1                         [clickable, blue]
    ItemType: 0x2 (2)                 [clickable, blue]
    Value: 42                         [clickable, blue]
```

### Packet Tree Display

```
Header:
  Sequence: 0x1234 (4660)             [clickable, blue]
  Flags: 0x4000 (16384)               [clickable, blue]
  Size: 448                           [clickable, blue]
Direction: "Recv"                     [clickable, blue]
```

## Benefits

1. **Better Protocol Understanding:** Users immediately see which values are IDs/flags (shown in hex)
2. **Quick Filtering:** One click to filter by any value
3. **Both Representations:** Hex values show decimal equivalent for reference
4. **Consistent UX:** All values are clickable and styled consistently
5. **Smart Detection:** Automatic format selection based on field semantics
6. **Backwards Compatible:** JSON serialization remains decimal (no breaking changes)

## Future Enhancements

Potential improvements:
1. Right-click context menu with "Copy hex", "Copy decimal" options
2. Ctrl+click on table cells to filter (in addition to tree view)
3. Custom field format rules loaded from configuration
4. Highlight filtered values in the tree
5. Multi-value filtering (AND/OR operations)

## Testing

**Build Status:** ✅ Clean build, no warnings

**Manual Testing Checklist:**
- [ ] Load example PCAP file
- [ ] View message in tree view
- [ ] Verify ObjectId shown as hex
- [ ] Verify OpCode shown as hex string (0xF7B0 format)
- [ ] Verify decimal values shown for counts/indices
- [ ] Click an ObjectId value
- [ ] Verify search box updates with hex value
- [ ] Verify messages filter correctly
- [ ] Type decimal value manually
- [ ] Verify it finds the same messages
- [ ] Test with various field types (IDs, flags, strings, numbers)

## Performance Considerations

- **No JSON Re-serialization:** Data displayed directly from serde_json::Value
- **Minimal Allocations:** String formatting only when rendering
- **Efficient Detection:** Field name checks use string comparison (fast)
- **Lazy Expansion:** Tree nodes only render when expanded
- **No Regex:** All string checks use simple character validation

## Accessibility

- **Keyboard Navigation:** Tree fully keyboard-navigable
- **Screen Readers:** Values announced as clickable links
- **Color Independence:** Hyperlink color respects theme (dark/light mode)
- **High Contrast:** Decimal values in grayed text maintain readability
