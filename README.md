# SuperDo

A sudoku puzzle solver written in Rust language.

## Download and run

Download from https://github.com/fengyc/superdo/releases

Or

    cargo install superdo

## Examples

1. sudoku with only one solution

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

        ---------
        748613925
        351928746
        926745813
        284359671
        597186432
        613472598
        435267189
        162894357

2. sudoku with multiple solutions

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

        ---------
        914623785
        652871394
        378459612
        723914856
        541268937
        869735421
        185347269
        297586143
        436192578
        ---------
        914623785
        652871394
        378459612
        723914856
        541268937
        869735421
        185347269
        297586143
        436192578
        
        ...

## License

MIT