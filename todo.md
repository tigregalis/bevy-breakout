gameplay

- [ ] implement change in direction based on position that ball strikes paddle
- [x] change collision detection - extrapolate next-frame position from bounce site
  - assuming collision on left
  - because the movement is linear
  - `(x_min_brick - x_max_ball_before)/(x_max_ball_after - x_max_ball_before)` =
    how much the ball did move / how much the ball should have moved =
    fraction of frame time it continues in its original direction
  - then switch the direction
  - then continue with the remainder of the frame time in the new direction
  - `x_min_brick` is the only thing we need to store, as we know the rest already
- [x] add game over, restart
- [ ] add levels
- [ ] add more brick types
- [ ] add power-ups that affect the ball and paddle

bugs

- [ ] when the ball is between the paddle and a side wall, it appears to escape
  - possibly fixed by setting up multiple collisions in one frame, or setting a precedence

aesthetics

- [x] implement rotation animation on the ball
- [x] add random colours for bricks
- [x] make the ball become "coloured" by the bricks it touches
- [x] make the paddle change colour to the ball
- [ ] make the random colours more pretty...
- [ ] add game sounds
- [ ] position win/lose message in center of screen (need to set up UI components)

code

- [ ] reorganise logic in systems
- [ ] reorganise code into modules and maybe plugins
