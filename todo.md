gameplay
- [ ] implement change in direction based on position that ball strikes paddle
- [ ] change collision detection - extrapolate next-frame position from bounce site
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

aesthetics
- [x] implement rotation animation on the ball
- [ ] add random colours for bricks
- [ ] make the ball become "coloured" by the bricks it touches
- [ ] make the paddle change colour to the ball
- [ ] add game sounds
- [ ] position win/lose message in center of screen

code
- [ ] reorganise logic in systems
- [ ] reorganise code into modules and maybe plugins