# Flee AI Test

This is a test project I've put together to let others try interacting with and AI for a game I'm working on.

## TODO

- [x] Implement "Fleeing" behavior (Make the agent head away from the player in the closest unobstructed direction)
  - [x] Implement basic [fleeing steering behavior](https://www.youtube.com/watch?v=Q4MU7pkDYmQ)
  - [x] Implement [directional weighting](https://youtu.be/6BrZryMz-ac?t=115)
    - [x] Use raycasts to check when a direction is "obstructed"
- [ ] Implement ["Wandering"](https://youtu.be/ujsR2vcJlLk) behavior (basically an idle animation)
- [ ] Blend between fleeing and wandering depending on player proximity (with LOS)
  - [ ] Desired direction controlled by fleeing vs wandering
  - [ ] Always use directional weighting to steer away from walls
  - [ ] If LOS, blend toward flee the closer the player gets
  - [ ] If no LOS, just wander
