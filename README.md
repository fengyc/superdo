# SuperDo

A sukudo resolver written in Rust language.

## Download and run

Download from https://github.com/fengyc/superdo/releases

## Examples

1. Sukudo with only one solution

    Input:

        040610925
        051000746
        926000813
        080050071
        090100032
        013470598
        000000189
        162800357
        809001264

    Output:

        Solved
        7 4 8 6 1 3 9 2 5
        3 5 1 9 2 8 7 4 6
        9 2 6 7 4 5 8 1 3
        2 8 4 3 5 9 6 7 1
        5 9 7 1 8 6 4 3 2
        6 1 3 4 7 2 5 9 8
        4 3 5 2 6 7 1 8 9
        1 6 2 8 9 4 3 5 7

2. Sukudo with mutiple solutions

    Input:

        000020085
        602070000
        300000010
        700904000
        040000030
        009005020
        005000260
        200086100
        030002070
    
    Output:

        Solved
        9 1 4 6 2 3 7 8 5 
        6 5 2 8 7 1 3 9 4 
        3 7 8 4 5 9 6 1 2 
        7 2 3 9 1 4 8 5 6 
        5 4 1 2 6 8 9 3 7 
        8 6 9 7 3 5 4 2 1 
        1 8 5 3 4 7 2 6 9 
        2 9 7 5 8 6 1 4 3 
        4 3 6 1 9 2 5 7 8 
        
        Solved
        9 1 4 6 2 3 7 8 5 
        6 5 2 8 7 1 3 9 4 
        3 7 8 4 5 9 6 1 2 
        7 2 3 9 1 4 8 5 6 
        5 4 1 2 6 8 9 3 7 
        8 6 9 7 3 5 4 2 1 
        1 8 5 3 4 7 2 6 9 
        2 9 7 5 8 6 1 4 3 
        4 3 6 1 9 2 5 7 8 
        
        ...

## License

MIT