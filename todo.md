# gameplay

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
- [ ] powerups (activatable)
  - [ ] ball passes through bricks
  - [ ] ball explodes on collision, destroying bricks in an area
  - [ ] ball explodes on collision, releasing smaller balls that have finite number of bounces
- [ ] lives
- [ ] blink/teleport using double-tap
- [ ] add more brick types
  - [ ] some bricks take multiple hits
  - [ ] some bricks release power ups
  - [ ] some bricks release 
- [ ] add power-ups that affect the ball and paddle
- [x] balls speed up as they hit bricks
- [ ] when the ball speeds up, the paddle speeds up as well
- [x] balls slow down as they hit walls
- [ ] when the ball slows down, the paddle slows down as well
- [ ] at the start, you can choose to release a ball from the paddle, and the direction it travels in
- [ ] score based on time elapsed (faster finish = higher score)

# logic

- [x] end game when there are zero bricks, not a max score

# new features

- [ ] menus
- [ ] level editor
- [ ] re-binding keys

# bugs

- [ ] when the ball is between the paddle and a side wall, it appears to escape
  - possibly caused by the fact it still counts as a collision, even though it passes through
  - possibly fixed by setting up multiple collisions in one frame, or setting a precedence
  - our new collision detection is too finicky - if something has already started passing through, it doesn't work
- [x] when the ball touches the side of the paddle, it changes colour when it shouldn't
- [ ] if two balls hit the same brick in the same frame, it grants two score
- [x] despawning balls grant score
- [ ] occasionally when you lose, and try to restart, it panics
  - thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: NoSuchEntity', C:\Users\Harimau\.cargo\git\checkouts\bevy-f7ffde730c324c74\3bc5e4c\crates\bevy_ecs\src\system\commands.rs:75:36
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace       
    error: process didn't exit successfully: `target\debug\my_bevy_game.exe` (exit code: 101)
  - something is wrong with the despawn system I'm guessing

# aesthetics

- [x] implement rotation animation on the ball
- [x] add random colours for bricks
- [x] make the ball become "coloured" by the bricks it touches
- [x] make the paddle change colour to the ball
- [ ] increase contrast of bricks from background
- [ ] make the random colours more pretty...
- [ ] add game sounds
- [ ] position win/lose message in center of screen
  - need to set up UI components properly
- [ ] instructional text "press R"
- [ ] speed of spin depending on speed of ball

# code

- [ ] reorganise logic into systems
- [ ] reorganise code into modules and maybe plugins
- [ ] rewrite to use entities instead of copying components
  - If `system_2` has a query like `transform_query: Query<&Transform>` then you can go
    `transform_query.get::<Transform>(entity)` to pull the Transform for that particular `Entity`."
