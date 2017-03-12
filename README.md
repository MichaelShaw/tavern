# tavern
Board game engine

Chip tune "hollow wanderer" is from 

https://www.youtube.com/watch?v=ajSIro2Kiug&index=22&list=PLye9mcKwe2zy3KW8uK_3F7HVMjJjdqSqU

[profile.dev]
opt-level = 3

# Santorini
- Implement negamax.
- Implement test suites that include test cases. Mild Player(0)/Player(1). Unavoidable winner for Player(0)/Player(1) (both sides). Avoidable winner for Player(0)/Player(1) both sides.
- Make iterative deepening a little smarter. If you reach a winning position terminate. If you reach a losing position terminate with result from last known pass if possible.
- Investigate performance cost of tracking move savings. Generic approach?
- Properly nest our state. "Game" should contain CPU as well as tentative stuff, proper nesting prevents enganglement.
- Add alpha beta pruning
- Add move ordering
- Add better heuristic

# Heuristic
- Height of pieces
- Height of squares available to move in (will reward high centre)

# Move Ordering
- Move up
- Build on a square one higher than an opponent 
- Move towards highest opponent

# AI 

### Naive Move Counting

#### From Start
    depth 0 moves -> 1 in 0.000s branch 1.0
    depth 1 moves -> 300 in 0.000s branch 300.0
    depth 2 moves -> 75900 in 0.002s branch 275.5
    depth 3 moves -> 4313232 in 0.109s branch 162.8
    depth 4 moves -> 237559488 in 5.846s branch 124.1
    
#### From move 2 (post start)
    depth 0 moves -> 1 in 0.000s branch 1.0
    depth 1 moves -> 33 in 0.000s branch 33.0
    depth 2 moves -> 996 in 0.000s branch 31.6
    depth 3 moves -> 43355 in 0.001s branch 35.1
    depth 4 moves -> 1814402 in 0.050s branch 36.7
    depth 5 moves -> 91341092 in 2.298s branch 39.1
    depth 6 moves -> 4467841671 in 109.096s branch 40.6
    
#### Add starting move symmetry checking
    depth 0 moves -> 1 in 0.000s branch 1.0
    depth 1 moves -> 85 in 0.000s branch 85.0
    depth 2 moves -> 20053 in 0.006s branch 141.6
    depth 3 moves -> 1148717 in 0.038s branch 104.7
    depth 4 moves -> 63167200 in 1.562s branch 89.2
    depth 5 moves -> 3841150351 in 91.991s branch 82.6
