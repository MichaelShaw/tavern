# tavern
Board game engine

Chip tune "hollow wanderer" is from 

https://www.youtube.com/watch?v=ajSIro2Kiug&index=22&list=PLye9mcKwe2zy3KW8uK_3F7HVMjJjdqSqU

# Santorini
- Add out of thread AI worker. Basically just stats producer to start.
- Add first heuristic measure (sum heights?)
- Add depth first search
- Add symmetry detection in initial placement (to reduce it x4 or so)

#Interesting move categories
- Move highest
- Build on a square one higher than an opponent 


# AI 

### Naive Move Counting

#### From Start

    depth 0 moves -> 1 in 0.000s
    depth 1 moves -> 300 in 0.000s
    depth 2 moves -> 75900 in 0.002s
    depth 3 moves -> 4313232 in 0.114s
    depth 4 moves -> 237559488 in 5.829s

#### From move 2 (post start)

    depth 0 moves -> 1 in 0.000s branch 1.0
    depth 1 moves -> 33 in 0.000s branch 33.0
    depth 2 moves -> 996 in 0.000s branch 31.6
    depth 3 moves -> 43355 in 0.001s branch 35.1
    depth 4 moves -> 1814402 in 0.050s branch 36.7
    depth 5 moves -> 91341092 in 2.298s branch 39.1
    depth 6 moves -> 4467841671 in 109.096s branch 40.6
