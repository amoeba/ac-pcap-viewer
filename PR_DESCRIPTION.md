# Add Custom AC-Aware JSON Tree Viewer with Hex Display and Click-to-Filter

## Summary

This PR implements intelligent hex/decimal display in the egui tree view with click-to-filter functionality, making it much easier to understand and navigate AC protocol data.

## What's New

### 🎯 Custom AC Protocol-Aware Tree Viewer

Replaced the generic `egui_json_tree` with a custom implementation that understands the AC protocol:

**Before:**
```
ObjectId: 2151762794
OpCode: "F7B0"
Sequence: 4660
```

**After:**
```
ObjectId: 0x80414B6A (2151762794)  ← clickable, blue
OpCode: 0xF7B0 (63408)             ← clickable, blue
Sequence: 0x1234 (4660)            ← clickable, blue
```

### ✨ Key Features

1. **Intelligent Hex Display**
   - Protocol-aware field detection (ObjectId, OpCode, Flags, SpellId, etc.)
   - Hex values shown with decimal in parentheses: `0xF7B0 (63408)`
   - Smart auto-detection for large numbers (>255) and ID fields
   - String hex values (like OpCode "F7B0") automatically formatted as `0xF7B0`

2. **Click-to-Filter**
   - Click any value in the tree to filter by it
   - Visual feedback: hyperlink color + pointing hand cursor
   - Tooltip: "Click to filter" on hover
   - Seamlessly integrates with existing filter system

3. **Backwards Compatible**
   - JSON serialization remains decimal (no breaking changes)
   - Existing filter system works unchanged
   - Can search by hex (`0xF7B0`) or decimal (`63408`) - both work

### 📁 Files Changed

- **New:** `crates/app/src/ui/ac_json_tree.rs` (364 lines)
  - Custom tree viewer with AC protocol awareness
  - DisplayFormat enum (Hex, Decimal, Auto)
  - Smart field detection based on AC protocol knowledge
  - Click handling and filter integration

- **Modified:** `crates/app/src/ui/detail_panel.rs`
  - Switched from `egui_json_tree::JsonTree` to `AcJsonTree`
  - Added filter click handling
  - Maintains same UI structure

- **Modified:** `crates/app/src/ui/mod.rs`
  - Added `ac_json_tree` module

- **New:** `HEX_DISPLAY_IMPLEMENTATION.md`
  - Comprehensive documentation of the feature

### 🔧 Technical Details

**Field Type Detection:**
- Automatically detects ID fields (ObjectId, SpellId, CasterId, etc.)
- Protocol fields (OpCode, Flags, Sequence)
- Any field ending in "Id" or "_id"
- Large numeric values (> 0xFFFF)

**Display Modes:**
- **Hex:** `0xF7B0 (63408)` - Primary hex, decimal in parens (grayed)
- **Decimal:** `42` - Simple decimal
- **Auto:** Decides based on value size (>255 → hex, ≤255 → decimal)

**User Experience:**
- All values are clickable (shown in blue hyperlink color)
- Pointer cursor on hover
- One-click filtering workflow
- Maintains existing keyboard navigation

### 🎨 Visual Improvements

- Clickable values use theme's hyperlink color
- Decimal equivalents shown in smaller, grayed text
- Boolean values color-coded (green/red)
- Consistent visual language throughout the tree

### 🧪 Testing

- ✅ Clean build (no warnings)
- ✅ Backwards compatible with existing filter system
- ✅ Works with both hex and decimal search input
- ✅ Tree expansion state preserved
- ✅ All field types properly detected and formatted

### 📊 User Benefits

1. **Better Protocol Understanding** - Immediately see which values are IDs/flags
2. **Quick Filtering** - One click to filter by any value
3. **Both Representations** - See hex and decimal simultaneously
4. **Discoverable** - Clear visual affordance makes feature obvious
5. **Flexible Search** - Can manually type hex or decimal

### 🔗 Related Issues

Addresses the need for better hex representation in the UI while maintaining the decimal JSON serialization format.

### 📝 Documentation

See `HEX_DISPLAY_IMPLEMENTATION.md` for comprehensive documentation including:
- Implementation details
- Code structure
- Examples
- Future enhancement ideas
- Performance considerations

---

## Example Workflow

1. User opens a message in the tree view
2. Sees `ObjectId: 0x80414B6A (2151762794)` in blue
3. Clicks on the value
4. Search box updates to `0x80414B6A`
5. Message list filters to show only messages with that ObjectId
6. User can also manually type `2151762794` (decimal) - both work!

## Screenshots

_Add screenshots showing the hex display and click-to-filter in action_
