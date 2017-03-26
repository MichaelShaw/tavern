# tavern
Board game engine

Check repeated hit rate for a given depth, how many exacts do we get.
Check if iterative deepending helps.

# Santorini
- Termination Cell + request tracking (seriously)
- Move ordering
- Better board representation (all maps/masks)
- Add something better than lerp for animation? Would a spring or bias curve be that much better?
- Request for RNG could be part of the AnalysisRequest ...
- Split off productivity library with group_by, contains etc? Hard to tell if this is a good idea or not.

## Move Ordering
- Move up
- Build adjacency/height
- Build on a square one higher than an opponent 
- Move towards highest opponent

## Value sooner victories more valuable
Perhaps we should reward faster victories than slower ones? Basically add remaining depth from to show how great it is?

The problem with this is it murders the transpotition table, as various states will be evaluated differently based on how far they are away .... the effects of which I'm not confident in.

## Heuristic
- Some notion of how many moveable squares are you closer to. So if they're in file 3, and you're in file 4, they get recognition for being closer to file 1.
- Alternate "Play it out" heuristic for when we've discovered that we can definitely lose.
- Remove trapped checking .... NeighbourHeuristic already scores trapped positions horifically.

### Other concerns
- Principal Variation
- Aspiration windows (alpha beta) based on last iterative deepening pass?
- Late Move Reduction
- "Quiet" moves, quinesence search.

## AI 

### Notes on our heuristic degradation

It's a matter of efficiency and bang for buck .... the heuristic can basically detect trapedness already.

We currently check for opponent trappedness when we evaluate the heuristic. This is currently the only method of checking that our last move secured a trapedness win.

It's interesting to basically lose 1 ply of search depth over this.

If we left this out, we'd still need to increase the extra ply to work out whether it's a trapping victory or not.

Running one extra ply, and leaving it out .... could still be more efficient. You run N+1 ply, you miss out on the opponents N+1 trap (you would if you ran N), and you get to run the heuristic at N+1.

Let's assume it's the opposite, the last move seen is them trapping you, due to complete lack of movement the heuristic score will be dreadful, basically near zero. (no adjacencies)

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
    
#### 16 Mar, unordered alpha beta
	MiniMax took 47 seconds 153,853,305 moves (3.3M/second) 36.6 avg branch
	NegaMax_AlphaBeta took 0.74 seconds 2,096,447 moves (2.8M/second) 20.9 avg branch
	NegaMax_AlphaBeta_Exp took 0.67 seconds 2,096,447 moves (3.1M/second) 20.9 avg branch

## Licensing Reminders

Chip tune "hollow wanderer" is from 

https://www.youtube.com/watch?v=ajSIro2Kiug&index=22&list=PLye9mcKwe2zy3KW8uK_3F7HVMjJjdqSqU
