# Task 5: Blood Vessel Harvesting

Blood vessels are the glucose source. Any hex adjacent to a vessel hex is a valid position for an astrocyte to harvest glucose.

## Implementation

When checking if a position is valid for astrocyte placement or when calculating harvest income:
1. Get the astrocyte's hex coordinates
2. Get all 6 neighbors using `hexNeighbors`
3. If any neighbor is a `vessel` hex, this position can harvest
4. Harvest rate scales with the number of adjacent vessel hexes (1 = base rate, 2+ = proportional bonus)

No special harvest node entities. The vessel terrain itself is the resource.

## Testing

Check several hexes adjacent to the main horizontal vessel. Verify they report as valid harvest positions. Check hexes at vessel bends/intersections — they should have higher harvest rates due to multiple adjacent vessel hexes. Check hexes far from any vessel — they should report as non-harvestable.
