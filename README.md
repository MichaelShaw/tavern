# tavern
Board game engine

# Santorini
- Add alpha beta pruning
- Add some form of move ordering?
- Add better heuristic
- Add iterative deepening based root move ordering (for alpha beta pruning)
- Adversarial heuristic testing
- Test MaxMax? (heuristic is relative? ... maybe it could work for 3+ player)
- Playout needs to be improved to include an equivalent of rollback.
- Add something better than lerp for animation.

## Heuristic
- Height of squares available to move in (will reward high centre)
- Alternate "Play it out" heuristic for when we've discovered that we can definitely lose.
- Heuristic when checking trapness should inform the heuristic which workers are free to move .... or will adjacency already cover this.

## Move Ordering
- Move up
- Build on a square one higher than an opponent 
- Move towards highest opponent

## AI 

### Notes on our heuristic degradation

We currently check for opponent trappedness when we evaluate the heuristic. This is currently the only method of checking that our last move secured a trapedness win.

It's interesting to basically lose 1 ply of search depth over this.

If we left this out, we'd still need to increase the extra ply to work out whether it's a trapping victory or not.

Running one extra ply, and leaving it out .... could still be more efficient. You run N+1 ply, you miss out on the opponents N+1 trap (you would if you ran N), and you get to run the heuristic at N+1.

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

## Licensing Reminders

Chip tune "hollow wanderer" is from 

https://www.youtube.com/watch?v=ajSIro2Kiug&index=22&list=PLye9mcKwe2zy3KW8uK_3F7HVMjJjdqSqU
