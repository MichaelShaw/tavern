# tavern
Board game engine


## AI 

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

