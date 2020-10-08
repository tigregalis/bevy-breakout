gameplay

- [x] implement change in direction based on position that ball strikes paddle
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
- [ ] additional balls
- [ ] balls of different sizes
- [ ] additional paddles
- [ ] paddles of different sizes
- [ ] powerups
- [ ] lives
- [ ] blink/teleport using double-tap
- [ ] add more brick types
- [ ] add power-ups that affect the ball and paddle
- [ ] balls speed up as they hit bricks - the paddle speeds up as well
- [ ] balls slow down as they hit walls - the paddle slows down as well
- [ ] at the start, you can choose to release a ball from the paddle

new features
- [ ] menus
- [ ] level editor

bugs

- [ ] when the ball is between the paddle and a side wall, it appears to escape
  - possibly caused by the fact it still counts as a collision, even though it passes through
  - possibly fixed by setting up multiple collisions in one frame, or setting a precedence
  - our new collision detection is too finicky - if something has already started passing through, it doesn't work
- [x] when the ball touches the side of the paddle, it changes colour when it shouldn't
- [ ] if two balls hit the same brick in the same frame, it grants two score

aesthetics

- [x] implement rotation animation on the ball
- [x] add random colours for bricks
- [x] make the ball become "coloured" by the bricks it touches
- [x] make the paddle change colour to the ball
- [ ] increase contrast of bricks from background
- [ ] make the random colours more pretty...
- [ ] add game sounds
- [ ] position win/lose message in center of screen (need to set up UI components)
- [ ] instructional text "press R"

code

- [ ] reorganise logic in systems
- [ ] reorganise code into modules and maybe plugins
- [ ] rewrite to use entities instead of copying components
  - Oh, I see. You can still use queries.
    If `system_2` has a query like `transform_query: Query<&Transform>` then you can go
    `transform_query.get::<Transform>(entity)` to pull the Transform for that particular `Entity`."