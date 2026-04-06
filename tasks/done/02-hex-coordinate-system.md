# Task 2: Hex Grid Coordinate System

Implement hex grid math for the pointy-top hex grid used by the map format.

## Parameters

- Hex radius: 24 units (center to vertex)
- Hex width: `sqrt(3) * 24 ≈ 41.6` units
- Hex height: `48` units
- Row height: `48 * 0.75 = 36` units
- Odd rows offset right by `width / 2`

## Functions to implement

**hexCenter(col, row):** Returns the world-space center position of a hex cell.
```
x = MARGIN_X + col * HEX_W + (row % 2) * HEX_W / 2
y = MARGIN_Y + row * ROW_H
```

**worldToHex(worldX, worldY):** Converts a world position to the nearest hex grid coordinates. Account for the odd-row offset.

**hexNeighbors(col, row):** Returns the 6 adjacent hex coordinates for a pointy-top grid.
```
Even row: (col+1,row), (col-1,row), (col,row-1), (col-1,row-1), (col,row+1), (col-1,row+1)
Odd row:  (col+1,row), (col-1,row), (col+1,row-1), (col,row-1), (col+1,row+1), (col,row+1)
```

**hexDistance(col1, row1, col2, row2):** Returns the hex grid distance (minimum number of hex steps) between two cells.

## Testing

Convert several known entity positions to hex coordinates and verify they match the `data-col`/`data-row` values from the SVG. Verify `hexNeighbors` returns 6 valid cells and `hexDistance` matches manual counts.
