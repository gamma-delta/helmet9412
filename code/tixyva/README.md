# tixyva

This is inspired by [tixy](tixy.land).

## Usage

Press Escape to go into editing mode. Write an Rhai function that returns an f64.

0 is black, 1 is full brightness. Positive numbers are lavender, negative numbers are green.

The variables `t`, `i`, `x`, `y`, and `a` are given to the function. `t` is the time, `i` is the index, `x` and `y` are the coordinates, `v` is the volume picked up from your microphone (from 0-3ish), and `a` is the previous value in that cell.

Press Enter to watch your function play!

Alternatively, press the number keys to see some pre-made patterns.
